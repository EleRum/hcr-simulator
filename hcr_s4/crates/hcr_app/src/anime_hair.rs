//! Anime Hair Studio OBJ loader.
//!
//! Parses multi-object OBJ files exported from Anime Hair Studio
//! (Blender addon). Each named object becomes a separate Bevy mesh
//! rendered with the hair alpha material.
//!
//! Coordinate conversion: OBJ Y-up → Bevy Z-up.

use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use std::collections::HashMap;
use hcr_core::hair::{CardSet, HairCard, CardVertex};
use hcr_core::types::Vec3 as CoreVec3;

/// One parsed hair object from the OBJ file.
pub struct AnimeHairPiece {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// All hair pieces loaded from an Anime Hair Studio OBJ.
pub struct AnimeHairSet {
    pub pieces: Vec<AnimeHairPiece>,
    pub mesh_handles: Vec<Handle<Mesh>>,
}

/// Build hcr_core CardSet from anime hair pieces.
/// Each piece → one HairCard, flat vertex layout.
/// Root vertices (top-most Z) are pinned.
pub fn build_card_set(set: &AnimeHairSet) -> CardSet {
    let mut cards = Vec::new();
    for piece in &set.pieces {
        let n = piece.positions.len();
        if n < 2 { continue; }

        let max_z = piece.positions.iter().map(|p| p[2]).fold(f32::MIN, f32::max);
        let mut rows = Vec::with_capacity(n);
        for vi in 0..n {
            let p = piece.positions[vi];
            let pos = CoreVec3::new(p[0], p[1], p[2]);
            let inv_mass = if (pos.z - max_z).abs() < 0.03 { 0.0 } else { 1.0 };
            rows.push(vec![CardVertex {
                pos, prev_pos: pos, inv_mass, active: true, uv: [0.0, 0.0],
            }]);
        }

        cards.push(HairCard {
            layer: 0,
            rows,
            name: piece.name.clone(),
            is_cut: false,
        });
    }

    CardSet { layers: vec![cards] }
}

impl AnimeHairSet {
    pub fn total_vertices(&self) -> usize {
        self.pieces.iter().map(|p| p.positions.len()).sum()
    }
}

/// Rebuild a Bevy Mesh from a HairCard (after cutting has modified vertex positions).
/// Reuses the original indices from the piece.
pub fn rebuild_mesh_from_card(card: &HairCard, original_indices: &[u32]) -> Mesh {
    let positions: Vec<[f32; 3]> = card.rows.iter().flat_map(|r| r.iter())
        .map(|v| [v.pos.x, v.pos.y, v.pos.z])
        .collect();

    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in original_indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let a = Vec3::from_array(positions[tri[0] as usize]);
        let b = Vec3::from_array(positions[tri[1] as usize]);
        let c = Vec3::from_array(positions[tri[2] as usize]);
        let n = (b - a).cross(c - a).normalize().to_array();
        for &vi in tri {
            let arr = &mut normals[vi as usize];
            arr[0] += n[0]; arr[1] += n[1]; arr[2] += n[2];
        }
    }
    for n in &mut normals {
        let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        if l > 0.0 { n[0] /= l; n[1] /= l; n[2] /= l; }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(original_indices.to_vec()))
}

/// Parse an Anime Hair Studio OBJ file.
/// Converts Y-up → Z-up, centers around origin.
pub fn load_anime_hair(path: &str, meshes: &mut Assets<Mesh>) -> AnimeHairSet {
    let text = std::fs::read_to_string(path).expect("Cannot read anime hair OBJ");

    // ── Parse all vertex data ──
    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_uvs: Vec<[f32; 2]> = Vec::new();

    // Per-object index buffers
    struct RawObj {
        name: String,
        pos_indices: Vec<u32>,  // 0-based position indices
        uv_indices: Vec<u32>,   // 0-based UV indices
    }
    let mut objects: Vec<RawObj> = Vec::new();
    let mut current: Option<RawObj> = None;

    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("o ") {
            if let Some(obj) = current.take() {
                if !obj.name.ends_with("_curve") && !obj.pos_indices.is_empty() {
                    objects.push(obj);
                }
            }
            current = Some(RawObj {
                name: l[2..].to_string(),
                pos_indices: Vec::new(),
                uv_indices: Vec::new(),
            });
        } else if l.starts_with("vt ") {
            let p: Vec<f32> = l[3..]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if p.len() >= 2 {
                all_uvs.push([p[0], p[1]]);
            }
        } else if l.starts_with("v ") {
            let p: Vec<f32> = l[2..]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if p.len() >= 3 {
                // Y-up OBJ → Z-up Bevy: (x, y, z) → (x, z, y)
                all_positions.push([p[0], p[2], p[1]]);
            }
        } else if l.starts_with("f ") {
            if let Some(ref mut obj) = current {
                // Parse format: f v1/vt1 v2/vt2 v3/vt3 [v4/vt4]
                let parts: Vec<&str> = l[2..].split_whitespace().collect();
                let mut pi: Vec<u32> = Vec::new();
                let mut ui: Vec<u32> = Vec::new();
                for part in &parts {
                    let mut s = part.split('/');
                    let p_idx: u32 = s.next().and_then(|x| x.parse().ok()).unwrap_or(0);
                    let t_idx: u32 = s.next().and_then(|x| x.parse().ok()).unwrap_or(0);
                    pi.push(if p_idx > 0 { p_idx - 1 } else { 0 });
                    ui.push(if t_idx > 0 { t_idx - 1 } else { 0 });
                }
                if pi.len() == 3 {
                    obj.pos_indices.extend_from_slice(&[pi[0], pi[1], pi[2]]);
                    obj.uv_indices.extend_from_slice(&[ui[0], ui[1], ui[2]]);
                } else if pi.len() >= 4 {
                    obj.pos_indices.extend_from_slice(&[pi[0], pi[1], pi[2]]);
                    obj.pos_indices.extend_from_slice(&[pi[0], pi[2], pi[3]]);
                    obj.uv_indices.extend_from_slice(&[ui[0], ui[1], ui[2]]);
                    obj.uv_indices.extend_from_slice(&[ui[0], ui[2], ui[3]]);
                }
            }
        }
    }
    // Last object
    if let Some(obj) = current {
        if !obj.name.ends_with("_curve") && !obj.pos_indices.is_empty() {
            objects.push(obj);
        }
    }

    // ── Center all positions ──
    let n = all_positions.len() as f32;
    let mut cx = 0.0f32; let mut cy = 0.0f32; let mut cz = 0.0f32;
    for p in &all_positions { cx += p[0]; cy += p[1]; cz += p[2]; }
    cx /= n; cy /= n; cz /= n;
    for p in &mut all_positions {
        p[0] -= cx; p[1] -= cy; p[2] -= cz;
    }
    info!("Anime hair OBJ: {} objects, {} verts, center=({:.3},{:.3},{:.3})",
        objects.len(), all_positions.len(), cx, cy, cz);

    // ── Reindex per-object: deduplicate (pos, uv) pairs ──
    let mut pieces = Vec::new();
    let mut mesh_handles = Vec::new();

    for obj in &objects {
        let mut local_map: HashMap<(u32, u32), u32> = HashMap::new();
        let mut local_positions: Vec<[f32; 3]> = Vec::new();
        let mut local_uvs: Vec<[f32; 2]> = Vec::new();
        let mut local_indices: Vec<u32> = Vec::new();

        assert_eq!(obj.pos_indices.len(), obj.uv_indices.len());

        for k in 0..obj.pos_indices.len() {
            let pi = obj.pos_indices[k];
            let ui = obj.uv_indices[k];
            let key = (pi, ui);
            let local_idx = *local_map.entry(key).or_insert_with(|| {
                let idx = local_positions.len() as u32;
                local_positions.push(all_positions[pi as usize]);
                local_uvs.push(all_uvs.get(ui as usize).copied().unwrap_or([0.0, 0.0]));
                idx
            });
            local_indices.push(local_idx);
        }

        // Compute normals
        let mut normals: Vec<[f32; 3]> = vec![[0.0; 3]; local_positions.len()];
        for tri in local_indices.chunks(3) {
            if tri.len() < 3 { continue; }
            let a = Vec3::from_array(local_positions[tri[0] as usize]);
            let b = Vec3::from_array(local_positions[tri[1] as usize]);
            let c = Vec3::from_array(local_positions[tri[2] as usize]);
            let n = (b - a).cross(c - a).normalize().to_array();
            for &vi in tri {
                let arr = &mut normals[vi as usize];
                arr[0] += n[0]; arr[1] += n[1]; arr[2] += n[2];
            }
        }
        for n in &mut normals {
            let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            if len > 0.0 { n[0] /= len; n[1] /= len; n[2] /= len; }
        }

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, local_positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, local_uvs.clone())
        .with_inserted_indices(Indices::U32(local_indices.clone()));

        let handle = meshes.add(mesh);
        mesh_handles.push(handle.clone());

        pieces.push(AnimeHairPiece {
            name: obj.name.clone(),
            positions: local_positions,
            indices: local_indices,
        });
    }

    AnimeHairSet { pieces, mesh_handles }
}
