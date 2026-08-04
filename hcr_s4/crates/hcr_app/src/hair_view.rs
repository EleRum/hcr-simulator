//! Hair card rendering system.
//!
//! Spawns renderable entities from the `CardAssets` resource,
//! applies per-layer materials with alpha masking, and manages
//! per-frame mesh updates when the M3 physics solver runs.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use crate::card_loader::CardAssets;

// ═══════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════

/// Per-layer tint colors (subtle variation for visual debugging).
const LAYER_COLORS: [[f32; 3]; 5] = [
    [0.10, 0.08, 0.06],  // L0 scalp   — dark brown
    [0.15, 0.10, 0.07],  // L1 inner   — brown
    [0.20, 0.13, 0.09],  // L2 middle  — medium brown
    [0.25, 0.16, 0.10],  // L3 outer   — lighter brown
    [0.18, 0.11, 0.07],  // L4 fringe  — warm brown
];

/// Alpha cutoff for Mask mode (discard fragments below this).
const ALPHA_CUTOFF: f32 = 0.4;

// ═══════════════════════════════════════════════════════════
// Material resource
// ═══════════════════════════════════════════════════════════

/// Material handles, one per layer.
#[derive(Resource)]
pub struct HairMaterials {
    pub handles: [Handle<StandardMaterial>; 5],
}

impl HairMaterials {
    pub fn new(materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>) -> Self {
        let hair_tex = create_hair_texture(images);

        let mut handles: [Handle<StandardMaterial>; 5] = Default::default();
        for li in 0..5 {
            handles[li] = materials.add(StandardMaterial {
                base_color: Color::srgb(
                    LAYER_COLORS[li][0],
                    LAYER_COLORS[li][1],
                    LAYER_COLORS[li][2],
                ),
                base_color_texture: Some(hair_tex.clone()),
                alpha_mode: AlphaMode::Mask(ALPHA_CUTOFF),
                perceptual_roughness: 0.7,
                double_sided: true, // cards are thin — render both sides
                ..default()
            });
        }

        Self { handles }
    }
}

// ═══════════════════════════════════════════════════════════
// Procedural hair strand texture
// ═══════════════════════════════════════════════════════════

/// Generate a simple 256×256 RGBA hair-strand-like texture.
///
/// Produces vertical hair lines with slight noise, giving the
/// alpha mask a plausible "hair strand" look without needing
/// a real atlas.  Can be swapped for an artist-authored atlas
/// later by replacing the image asset.
fn create_hair_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = 256u32;
    let mut pixels: Vec<u8> = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            // Vertical hair strands: alpha varies with x in a stripe pattern
            let fx = x as f32;
            let fy = y as f32;
            let s = size as f32;

            // Multiple overlapping sine waves for strand density
            let strand0 = ((fx * 12.0 / s) * std::f32::consts::PI * 2.0).sin().abs();
            let strand1 = ((fx * 20.0 / s + 3.0) * std::f32::consts::PI * 2.0).sin().abs();
            let strand2 = ((fx * 35.0 / s + 7.0) * std::f32::consts::PI * 2.0).sin().abs();

            let density = (strand0 * 0.5 + strand1 * 0.3 + strand2 * 0.2).clamp(0.0, 1.0);

            // Threshold for strand visibility
            let alpha = if density > 0.85 { 1.0 } else { 0.0 };

            // Color: off-white with slight warmth
            let r = (220.0 + (fy / s) * 35.0) as u8;
            let g = (180.0 + (fy / s) * 40.0) as u8;
            let b = (140.0 + (fy / s) * 30.0) as u8;
            let a = (alpha * 255.0) as u8;

            pixels.extend_from_slice(&[r, g, b, a]);
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        pixels,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    images.add(image)
}

// ═══════════════════════════════════════════════════════════
// Spawn system
// ═══════════════════════════════════════════════════════════

/// Marker component for hair card entities.
#[derive(Component)]
pub struct HairCardTag {
    pub layer: u8,
    pub card_index: u32,
}

/// Spawn all hair card entities from loaded assets.
/// Runs once in Startup after CardAssets is inserted.
pub fn spawn_hair_cards(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    hair_materials: Res<HairMaterials>,
) {
    info!("Spawning hair cards...");

    for li in 0..5 {
        let cards = &card_assets.loaded.layers[li];
        let handles = &card_assets.loaded.mesh_handles[li];
        let material = hair_materials.handles[li].clone();

        for (ci, (_card, mesh_handle)) in cards.iter().zip(handles.iter()).enumerate() {
            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(), // Card verts already in world space from Blender
                HairCardTag {
                    layer: li as u8,
                    card_index: ci as u32,
                },
            ));
        }

        info!("  Layer {li}: {} cards spawned", cards.len());
    }

    info!("Total hair card entities: {} (5 draw calls via material batching)", {
        let mut total = 0u32;
        for li in 0..5 {
            total += card_assets.loaded.layers[li].len() as u32;
        }
        total
    });
}
