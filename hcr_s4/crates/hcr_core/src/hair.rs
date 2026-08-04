//! Hair card data types — card = independent quad strip mesh.
//!
//! Each card is a small grid (1–2 cols × 4–8 rows),
//! rooted at the scalp (row 0 pinned, `inv_mass = 0`).
//! Cards are loaded from Blender-exported glTF, or generated
//! procedurally as a fallback.

use crate::types::Vec3;
use crate::head::HeadMesh;

/// A single vertex in a hair card.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CardVertex {
    pub pos: Vec3,
    pub prev_pos: Vec3,
    /// 0 = pinned (root), 1 = free
    pub inv_mass: f32,
    pub active: bool,
    /// UV for texture atlas lookup.
    pub uv: [f32; 2],
}

/// One independent hair card (quad strip mesh).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HairCard {
    pub layer: u8,
    /// Grid: rows[ri][ci].
    /// Row 0 = root (scalp), last row = tip.
    pub rows: Vec<Vec<CardVertex>>,
    /// Name from Blender object, e.g. "CARD_L3_042".
    pub name: String,
    /// Whether tip has been cut (for visual tinting).
    pub is_cut: bool,
}

impl HairCard {
    /// Number of active vertices in this card.
    pub fn active_count(&self) -> usize {
        self.rows.iter().flatten().filter(|v| v.active).count()
    }

    /// Total vertices.
    pub fn total_vertices(&self) -> usize {
        self.rows.len() * self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    /// Check if any vertex in the last active row is within `r2` of `center`.
    pub fn tip_in_sphere(&self, center: Vec3, r2: f32) -> Option<usize> {
        for ri in (0..self.rows.len()).rev() {
            if self.rows[ri].iter().any(|v| v.active) {
                let any_in = self.rows[ri].iter()
                    .any(|v| v.active && (v.pos - center).length_squared() < r2);
                return if any_in { Some(ri) } else { None };
            }
        }
        None
    }

    /// Find the last row index that has active vertices inside the sphere.
    /// Returns the row index from tip direction (largest ri first).
    pub fn tipmost_row_in_sphere(&self, center: Vec3, r2: f32) -> Option<usize> {
        for ri in (0..self.rows.len()).rev() {
            let all_in = self.rows[ri].iter()
                .filter(|v| v.active)
                .all(|v| (v.pos - center).length_squared() < r2);
            if all_in && self.rows[ri].iter().any(|v| v.active) {
                return Some(ri);
            }
        }
        None
    }

    /// Deactivate rows from `from_row` to tip. Returns the deactivated vertices.
    pub fn cut_tail(&mut self, from_row: usize) -> Vec<Vec3> {
        let mut debris = Vec::new();
        for ri in from_row..self.rows.len() {
            for v in &mut self.rows[ri] {
                if v.active {
                    debris.push(v.pos);
                    v.active = false;
                }
            }
        }
        if from_row > 0 {
            self.is_cut = true;
        }
        debris
    }
}

/// All hair cards for one hairstyle, organized by layer.
#[derive(Clone, Debug)]
pub struct CardSet {
    /// Cards ordered outer (layer 0) → inner (layer N).
    pub layers: Vec<Vec<HairCard>>,
}

impl CardSet {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn total_cards(&self) -> usize {
        self.layers.iter().map(|l| l.len()).sum()
    }

    pub fn total_active_vertices(&self) -> usize {
        self.layers.iter().flatten().map(|c| c.active_count()).sum()
    }

    /// All cards from outer to inner, with layer index.
    pub fn all_cards(&self) -> impl Iterator<Item = (usize, &HairCard)> {
        self.layers.iter().enumerate()
            .flat_map(|(li, layer)| layer.iter().map(move |c| (li, c)))
    }

    /// Mutable access to all cards.
    pub fn all_cards_mut(&mut self) -> impl Iterator<Item = (usize, &mut HairCard)> {
        self.layers.iter_mut().enumerate()
            .flat_map(|(li, layer)| layer.iter_mut().map(move |c| (li, c)))
    }
}

/// Generate procedural cards from the existing raycast-based approach.
/// Fallback when Blender assets aren't available yet.
pub fn generate_procedural_cards(head: &HeadMesh, n_layers: usize, cards_per_layer: usize) -> CardSet {
    // Simple procedural fallback — create 1-wide cards by raycasting
    // in bands around the head.
    let mut layers = Vec::new();

    for li in 0..n_layers {
        let t = li as f32 / n_layers.max(1) as f32;
        let theta_max = 0.3 + t * 1.1; // increase with layer
        let length = 0.08 + t * 0.42;
        let n_rows = 4 + (t * 4.0) as usize;
        let mut cards = Vec::new();

        for ci in 0..cards_per_layer {
            let phi = (ci as f32 / cards_per_layer as f32) * 2.0 * std::f32::consts::PI;
            let mut rows: Vec<Vec<CardVertex>> = Vec::new();

            for ri in 0..=n_rows {
                let theta = theta_max * (ri as f32 / n_rows.max(1) as f32);
                let mut col_verts = Vec::new();

                if let Some((root_pos, normal)) = head.raycast(theta, phi) {
                    let depth = length * (ri as f32 / n_rows.max(1) as f32);
                    let pos = root_pos + normal * depth;
                    col_verts.push(CardVertex {
                        pos,
                        prev_pos: pos,
                        inv_mass: if ri == 0 { 0.0 } else { 1.0 },
                        active: true,
                        uv: [phi / (2.0 * std::f32::consts::PI), theta / std::f32::consts::PI],
                    });
                }

                if !col_verts.is_empty() {
                    rows.push(col_verts);
                }
            }

            if rows.len() >= 2 {
                cards.push(HairCard {
                    layer: li as u8,
                    rows,
                    name: format!("CARD_L{}_{:03}", li, ci),
                    is_cut: false,
                });
            }
        }

        layers.push(cards);
    }

    CardSet { layers }
}
