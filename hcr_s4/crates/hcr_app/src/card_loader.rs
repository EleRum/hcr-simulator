//! GLB parser and card data extractor.
//!
//! Loads `hair_L0..4.glb`, parses the binary glTF 2.0 format,
//! extracts positions / normals / UVs / indices for each named
//! card mesh, and builds both Bevy `Mesh` assets and the
//! hcr_core `CardSet` data structure (used by M3 physics).
//!
//! The Bevy glTF loader is sidestepped here because we need
//! CPU-side vertex data for CardSet construction.  A raw GLB
//! parser is compact and straightforward for our needs (every
//! file is ≤ 48 meshes, embedded BIN chunk, no extensions).

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use hcr_core::hair::{CardSet, HairCard, CardVertex};
use hcr_core::types::Vec3 as CoreVec3;

// ═══════════════════════════════════════════════════════════
// Raw GLB data structures (JSON mirrors)
// ═══════════════════════════════════════════════════════════

/// Minimal glTF JSON — only the fields we actually read.
#[derive(serde::Deserialize)]
struct GltfRoot {
    meshes: Vec<GltfMesh>,
    nodes: Vec<GltfNode>,
    accessors: Vec<GltfAccessor>,
    #[serde(rename = "bufferViews")]
    buffer_views: Vec<GltfBufferView>,
}

#[derive(serde::Deserialize)]
struct GltfMesh {
    name: Option<String>,
    primitives: Vec<GltfPrimitive>,
}

#[derive(serde::Deserialize)]
struct GltfPrimitive {
    attributes: HashMap<String, usize>,
    indices: Option<usize>,
}

#[derive(serde::Deserialize)]
struct GltfNode {
    name: Option<String>,
    mesh: Option<usize>,
    children: Option<Vec<usize>>,
}

#[derive(serde::Deserialize)]
struct GltfAccessor {
    #[serde(rename = "bufferView")]
    buffer_view: Option<usize>,
    #[serde(rename = "byteOffset", default)]
    byte_offset: usize,
    #[serde(rename = "componentType")]
    component_type: u32,
    count: usize,
    #[serde(rename = "type")]
    accessor_type: String,
}

#[derive(serde::Deserialize)]
struct GltfBufferView {
    buffer: usize,
    #[serde(rename = "byteOffset", default)]
    byte_offset: usize,
    #[serde(rename = "byteLength")]
    byte_length: usize,
}

// ═══════════════════════════════════════════════════════════
// Parsed card data
// ═══════════════════════════════════════════════════════════

/// One card mesh, extracted from a GLB file.
pub struct RawCard {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub layer: u8,
    pub card_index: u32,
}

/// All cards loaded from all 5 layer files.
pub struct LoadedCards {
    /// Per-layer raw cards.
    pub layers: [Vec<RawCard>; 5],
    /// Bevy mesh handles, per-layer.
    pub mesh_handles: [Vec<Handle<Mesh>>; 5],
    /// hcr_core CardSet for M3 physics.
    pub card_set: CardSet,
}

impl RawCard {
    /// Offset all vertex positions by `(dx, dy, dz)`.
    pub fn offset_positions(&mut self, offset: [f32; 3]) {
        for p in &mut self.positions {
            p[0] += offset[0];
            p[1] += offset[1];
            p[2] += offset[2];
        }
    }
}

// ═══════════════════════════════════════════════════════════
// GLB binary parser
// ═══════════════════════════════════════════════════════════

/// Parse a raw GLB byte buffer into a list of `RawCard` meshes.
pub fn parse_glb(data: &[u8]) -> Result<Vec<RawCard>, String> {
    if data.len() < 20 {
        return Err("GLB too short".into());
    }
    // Header
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != 0x46546C67 {
        return Err(format!("Bad magic: 0x{magic:08X}, expected 0x46546C67 (glTF)"));
    }
    let _version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let _total_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

    let mut offset: usize = 12;
    let mut json_bytes: Option<&[u8]> = None;
    let mut bin_bytes: Option<&[u8]> = None;

    while offset + 8 <= data.len() {
        let chunk_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
        let chunk_type = u32::from_le_bytes([
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
        offset += 8;

        if offset + chunk_len > data.len() {
            return Err("Chunk overflows file".into());
        }
        let chunk_data = &data[offset..offset + chunk_len];

        match chunk_type {
            0x4E4F534A => json_bytes = Some(chunk_data),   // JSON
            0x004E4942 => bin_bytes = Some(chunk_data),     // BIN
            _ => { /* skip unknown chunks */ }
        }
        offset += chunk_len;
    }

    let json_data = json_bytes.ok_or("Missing JSON chunk")?;
    let bin_data = bin_bytes.ok_or("Missing BIN chunk")?;

    let root: GltfRoot = serde_json::from_slice(json_data)
        .map_err(|e| format!("JSON parse error: {e}"))?;

    // ── Resolve each mesh ──
    let mut cards = Vec::new();

    for node in &root.nodes {
        let Some(mesh_idx) = node.mesh else { continue };
        let Some(name) = &node.name else { continue };

        let mesh = &root.meshes[mesh_idx];
        let prim = mesh.primitives.first()
            .ok_or_else(|| format!("Mesh {name} has no primitives"))?;

        // Resolve attribute accessors
        let pos_acc_idx = prim.attributes.get("POSITION")
            .ok_or_else(|| format!("{name}: missing POSITION"))?;
        let norm_acc_idx = prim.attributes.get("NORMAL")
            .ok_or_else(|| format!("{name}: missing NORMAL"))?;
        let uv_acc_idx = prim.attributes.get("TEXCOORD_0")
            .ok_or_else(|| format!("{name}: missing TEXCOORD_0"))?;
        let idx_acc_idx = prim.indices
            .ok_or_else(|| format!("{name}: missing indices"))?;

        let positions = read_accessor_vec3(&root, bin_data, *pos_acc_idx)?;
        let normals = read_accessor_vec3(&root, bin_data, *norm_acc_idx)?;
        let uvs = read_accessor_vec2(&root, bin_data, *uv_acc_idx)?;
        let indices = read_accessor_indices(&root, bin_data, idx_acc_idx)?;

        let (layer, card_index) = parse_card_name(name)?;

        cards.push(RawCard {
            name: name.clone(),
            positions,
            normals,
            uvs,
            indices,
            layer,
            card_index,
        });
    }

    // Sort by layer, then card index
    cards.sort_by(|a, b| a.layer.cmp(&b.layer).then(a.card_index.cmp(&b.card_index)));

    Ok(cards)
}

// ═══════════════════════════════════════════════════════════
// Accessor resolvers
// ═══════════════════════════════════════════════════════════

/// Resolve accessor data from the BIN chunk.
/// Computes the correct byte range from accessor count × element size,
/// NOT the full bufferView length (which may be shared by multiple accessors).
fn resolve_accessor_bytes<'a>(
    root: &GltfRoot,
    bin: &'a [u8],
    acc_idx: usize,
) -> Result<&'a [u8], String> {
    let acc = &root.accessors[acc_idx];
    let bv_idx = acc.buffer_view.ok_or("Accessor missing bufferView")?;
    let bv = &root.buffer_views[bv_idx];
    if bv.buffer != 0 {
        return Err(format!("External buffer ref {} unsupported", bv.buffer));
    }

    let comp_size: usize = match acc.component_type {
        5126 => 4, // FLOAT
        5125 => 4, // UNSIGNED_INT
        5123 => 2, // UNSIGNED_SHORT
        5121 => 1, // UNSIGNED_BYTE
        _ => return Err(format!("Unknown componentType {}", acc.component_type)),
    };
    let num_comp: usize = match acc.accessor_type.as_str() {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        _ => return Err(format!("Unknown accessor type {}", acc.accessor_type)),
    };
    let element_size = comp_size * num_comp;
    let data_len = acc.count * element_size;

    let start = bv.byte_offset + acc.byte_offset;
    let end = start + data_len;

    if end > bin.len() {
        return Err(format!("Accessor {acc_idx} overflows BIN chunk"));
    }
    Ok(&bin[start..end])
}

fn read_accessor_vec3(root: &GltfRoot, bin: &[u8], acc_idx: usize) -> Result<Vec<[f32; 3]>, String> {
    let acc = &root.accessors[acc_idx];
    if acc.accessor_type != "VEC3" || acc.component_type != 5126 {
        return Err(format!("Expected VEC3/FLOAT for accessor {acc_idx}, got {}/{}",
            acc.accessor_type, acc.component_type));
    }
    let bytes = resolve_accessor_bytes(root, bin, acc_idx)?;
    let floats: Vec<f32> = bytes_to_floats(bytes);
    let mut out = Vec::with_capacity(acc.count);
    for i in 0..acc.count {
        let base = i * 3;
        out.push([floats[base], floats[base+1], floats[base+2]]);
    }
    Ok(out)
}

fn read_accessor_vec2(root: &GltfRoot, bin: &[u8], acc_idx: usize) -> Result<Vec<[f32; 2]>, String> {
    let acc = &root.accessors[acc_idx];
    if acc.accessor_type != "VEC2" || acc.component_type != 5126 {
        return Err(format!("Expected VEC2/FLOAT for accessor {acc_idx}, got {}/{}",
            acc.accessor_type, acc.component_type));
    }
    let bytes = resolve_accessor_bytes(root, bin, acc_idx)?;
    let floats: Vec<f32> = bytes_to_floats(bytes);
    let mut out = Vec::with_capacity(acc.count);
    for i in 0..acc.count {
        let base = i * 2;
        out.push([floats[base], floats[base+1]]);
    }
    Ok(out)
}

fn read_accessor_indices(root: &GltfRoot, bin: &[u8], acc_idx: usize) -> Result<Vec<u32>, String> {
    let acc = &root.accessors[acc_idx];
    if acc.accessor_type != "SCALAR" {
        return Err(format!("Expected SCALAR for indices accessor {acc_idx}"));
    }
    let bytes = resolve_accessor_bytes(root, bin, acc_idx)?;
    match acc.component_type {
        5121 => Ok(bytes.iter().map(|&b| b as u32).collect()), // UNSIGNED_BYTE
        5123 => { // UNSIGNED_SHORT
            let shorts: Vec<u16> = bytes_to_shorts(bytes);
            Ok(shorts.into_iter().map(|s| s as u32).collect())
        }
        5125 => { // UNSIGNED_INT
            let ints: Vec<u32> = bytes_to_u32s(bytes);
            Ok(ints)
        }
        _ => Err(format!("Unsupported index component type: {}", acc.component_type)),
    }
}

// ═══════════════════════════════════════════════════════════
// Byte conversion helpers
// ═══════════════════════════════════════════════════════════

fn bytes_to_floats(bytes: &[u8]) -> Vec<f32> {
    assert!(bytes.len() % 4 == 0);
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    out
}

fn bytes_to_shorts(bytes: &[u8]) -> Vec<u16> {
    assert!(bytes.len() % 2 == 0);
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        out.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    out
}

fn bytes_to_u32s(bytes: &[u8]) -> Vec<u32> {
    assert!(bytes.len() % 4 == 0);
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    out
}

// ═══════════════════════════════════════════════════════════
// Name parsing
// ═══════════════════════════════════════════════════════════

/// Parse `CARD_L{layer}_{idx:03d}` → `(layer, index)`.
fn parse_card_name(name: &str) -> Result<(u8, u32), String> {
    // Expected format: "CARD_L3_042"
    if !name.starts_with("CARD_L") {
        return Err(format!("Bad card name: {name}"));
    }
    let rest = &name[6..]; // after "CARD_L"
    let parts: Vec<&str> = rest.split('_').collect();
    if parts.len() != 2 {
        return Err(format!("Bad card name format: {name}"));
    }
    let layer: u8 = parts[0].parse()
        .map_err(|_| format!("Bad layer in name: {name}"))?;
    let idx: u32 = parts[1].parse()
        .map_err(|_| format!("Bad index in name: {name}"))?;
    Ok((layer, idx))
}

// ═══════════════════════════════════════════════════════════
// Bevy Mesh builder
// ═══════════════════════════════════════════════════════════

pub fn card_to_bevy_mesh(card: &RawCard) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, card.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, card.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, card.uvs.clone())
    .with_inserted_indices(Indices::U32(card.indices.clone()))
}

// ═══════════════════════════════════════════════════════════
// CardSet builder
// ═══════════════════════════════════════════════════════════

/// Build an hcr_core `CardSet` from raw card references.
pub fn build_card_set_from_refs(cards: &[&RawCard]) -> CardSet {
    let raw_cards: Vec<&RawCard> = cards.to_vec();
    build_card_set_inner(raw_cards.iter().map(|c| *c))
}

/// Build from owned RawCard slice.
pub fn build_card_set(cards: &[RawCard]) -> CardSet {
    build_card_set_inner(cards.iter())
}

/// Shared implementation.
/// Builds CardSet with flat vertex layout (one vertex per row, 1 col)
/// because Blender exports unwelded per-quad vertices.
/// Edge connectivity for PBD is derived from the index buffer edges.
fn build_card_set_inner<'a>(cards: impl Iterator<Item = &'a RawCard>) -> CardSet {
    let mut layers: Vec<Vec<HairCard>> = vec![Vec::new(); 5];

    for raw in cards {
        let li = raw.layer as usize;
        if li >= layers.len() {
            continue;
        }

        // Use ALL vertices as-is from the GLB.
        // Each vertex is its own "row" with 1 column.
        // Pinned: first 2 verts (root of the card).
        let n = raw.positions.len();
        let mut card_rows: Vec<Vec<CardVertex>> = Vec::with_capacity(n);

        for vi in 0..n {
            let pos_arr = raw.positions[vi];
            let uv_arr = raw.uvs.get(vi).copied().unwrap_or([0.0; 2]);
            let pos = CoreVec3::new(pos_arr[0], pos_arr[1], pos_arr[2]);
            // Pin root vertices (first few, near scalp)
            let inv_mass = if vi < 2 { 0.0 } else { 1.0 };
            card_rows.push(vec![CardVertex {
                pos,
                prev_pos: pos,
                inv_mass,
                active: true,
                uv: uv_arr,
            }]);
        }

        layers[li].push(HairCard {
            layer: raw.layer,
            rows: card_rows,
            name: raw.name.clone(),
            is_cut: false,
        });
    }

    CardSet { layers }
}

// ═══════════════════════════════════════════════════════════
// Full loader
// ═══════════════════════════════════════════════════════════

const LAYER_FILES: [&str; 5] = [
    "hair/hair_L0.glb",
    "hair/hair_L1.glb",
    "hair/hair_L2.glb",
    "hair/hair_L3.glb",
    "hair/hair_L4.glb",
];

/// Load all 5 hair layer GLBs from the assets directory.
/// `center_offset` is subtracted from all positions to align cards
/// with the centered head mesh (both rendering and simulation).
/// Returns `LoadedCards` with meshes and CardSet ready.
pub fn load_all_cards(asset_server: &AssetServer, center_offset: [f32; 3]) -> LoadedCards {
    let mut all_raw: [Vec<RawCard>; 5] = Default::default();
    let mut all_mesh_handles: [Vec<Handle<Mesh>>; 5] = Default::default();

    for li in 0..5 {
        let path = format!("assets/{}", LAYER_FILES[li]);
        let data = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Cannot read {path}: {e}"));
        let mut cards = parse_glb(&data)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

        // Offset positions to match centered head mesh
        for card in &mut cards {
            card.offset_positions(center_offset);
        }

        // Create Bevy Mesh for each card
        let mut handles = Vec::with_capacity(cards.len());
        for card in &cards {
            let mesh = card_to_bevy_mesh(card);
            handles.push(asset_server.add(mesh));
        }

        all_raw[li] = cards;
        all_mesh_handles[li] = handles;
    }

    // Build CardSet from all raw cards
    let all_cards: Vec<&RawCard> = all_raw.iter().flatten().collect();
    let card_set = build_card_set_from_refs(&all_cards);

    LoadedCards {
        layers: all_raw,
        mesh_handles: all_mesh_handles,
        card_set,
    }
}

// ═══════════════════════════════════════════════════════════
// Resource wrapper
// ═══════════════════════════════════════════════════════════

/// Bevy resource: all hair cards loaded and parsed.
#[derive(Resource)]
pub struct CardAssets {
    pub loaded: LoadedCards,
}

impl CardAssets {
    pub fn new(asset_server: &AssetServer, center_offset: [f32; 3]) -> Self {
        Self { loaded: load_all_cards(asset_server, center_offset) }
    }
}
