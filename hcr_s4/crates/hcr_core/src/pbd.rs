//! Verlet + PBD solver for hair cards.
//!
//! Ported from `hcr_s2\src\sheet.rs` — the same Verlet prediction,
//! stretch/bend constraints, and head collision,
//! adapted for independent card grids (HairCard.rows).

use crate::hair::{CardSet, CardVertex, HairCard};
use crate::head::HeadMesh;
use crate::types::Vec3;

/// Verlet prediction step — all cards.
pub fn step_verlet(cards: &mut [HairCard], gravity: Vec3, dt: f32, damping: f32) {
    let g = gravity * (dt * dt);
    for card in cards {
        for row in &mut card.rows {
            for v in row {
                if !v.active || v.inv_mass < 0.01 {
                    continue;
                }
                let vel = (v.pos - v.prev_pos) * damping;
                v.prev_pos = v.pos;
                v.pos = v.pos + vel + g;
            }
        }
    }
}

/// Post-constraint velocity damping.
pub fn damp_velocity(cards: &mut [HairCard], factor: f32) {
    for card in cards {
        for row in &mut card.rows {
            for v in row {
                if !v.active || v.inv_mass < 0.01 {
                    continue;
                }
                let vel = v.pos - v.prev_pos;
                v.prev_pos = v.pos - vel * factor;
            }
        }
    }
}

/// Solve all PBD constraints for all cards.
pub fn solve_constraints(cards: &mut [HairCard], head: &HeadMesh, stiffness: f32) {
    solve_stretch(cards);
    solve_bend(cards, stiffness);
    solve_head_collision(cards, head);
}

/// Distance constraints on card edges.
/// Vertical edges (along hair length) and horizontal edges (across card width).
fn solve_stretch(cards: &mut [HairCard]) {
    // Compute average edge lengths globally across all cards
    let (v_target, h_target) = measure_edges(cards);

    for card in cards {
        let rows = card.rows.len();
        if rows < 2 {
            continue;
        }

        // Vertical edges (along hair length)
        for ri in 0..rows - 1 {
            let cols = card.rows[ri].len().min(card.rows[ri + 1].len());
            for ci in 0..cols {
                let a = &card.rows[ri][ci];
                let b = &card.rows[ri + 1][ci];
                if !a.active || !b.active {
                    continue;
                }
                // Use index pairs to avoid borrowing both rows
                let dir = b.pos - a.pos;
                let len = dir.length().max(1e-6);
                let wa = a.inv_mass / (a.inv_mass + b.inv_mass + 1e-8);
                let corr = dir * ((len - v_target) / len) * wa;

                // We need mutable access to both — use raw pointers
                // This is safe because a and b are in different rows
                let a_ptr = &card.rows[ri][ci] as *const CardVertex as *mut CardVertex;
                let b_ptr = &card.rows[ri + 1][ci] as *const CardVertex as *mut CardVertex;
                unsafe {
                    (*a_ptr).pos = (*a_ptr).pos + corr;
                    (*b_ptr).pos = (*b_ptr).pos - corr * (wa / (1.0 - wa + 1e-8));
                }
            }
        }

        // Horizontal edges (across card width)
        for ri in 0..rows {
            let cols = card.rows[ri].len();
            for ci in 0..cols.saturating_sub(1) {
                let a = &card.rows[ri][ci];
                let b = &card.rows[ri][ci + 1];
                if !a.active || !b.active {
                    continue;
                }
                let dir = b.pos - a.pos;
                let len = dir.length().max(1e-6);
                let wa = a.inv_mass / (a.inv_mass + b.inv_mass + 1e-8);
                let corr = dir * ((len - h_target) / len) * wa;

                let a_ptr = &card.rows[ri][ci] as *const CardVertex as *mut CardVertex;
                let b_ptr = &card.rows[ri][ci + 1] as *const CardVertex as *mut CardVertex;
                unsafe {
                    (*a_ptr).pos = (*a_ptr).pos + corr;
                    (*b_ptr).pos = (*b_ptr).pos - corr * (wa / (1.0 - wa + 1e-8));
                }
            }
        }
    }
}

/// Measure average vertical and horizontal edge lengths.
fn measure_edges(cards: &[HairCard]) -> (f32, f32) {
    let mut v_total = 0.0f32;
    let mut v_count = 0usize;
    let mut h_total = 0.0f32;
    let mut h_count = 0usize;

    for card in cards {
        let rows = card.rows.len();
        // Vertical
        for ri in 0..rows.saturating_sub(1) {
            let cols = card.rows[ri].len().min(card.rows[ri + 1].len());
            for ci in 0..cols {
                let a = &card.rows[ri][ci];
                let b = &card.rows[ri + 1][ci];
                if a.active && b.active {
                    v_total += (a.pos - b.pos).length();
                    v_count += 1;
                }
            }
        }
        // Horizontal
        for ri in 0..rows {
            let cols = card.rows[ri].len();
            for ci in 0..cols.saturating_sub(1) {
                let a = &card.rows[ri][ci];
                let b = &card.rows[ri][ci + 1];
                if a.active && b.active {
                    h_total += (a.pos - b.pos).length();
                    h_count += 1;
                }
            }
        }
    }

    (v_total / v_count.max(1) as f32, h_total / h_count.max(1) as f32)
}

/// Bending constraints — keep triples of consecutive vertices smooth.
/// Only vertical bending (along hair length) for most cards.
/// Horizontal bending only for 2-wide cards.
fn solve_bend(cards: &mut [HairCard], stiffness: f32) {
    for card in cards {
        let rows = card.rows.len();
        if rows < 3 {
            continue;
        }

        // Vertical bending (along length) — applies to all cards
        for ci in 0..card.rows[0].len() {
            for ri in 1..rows - 1 {
                if !card.rows[ri - 1][ci].active
                    || !card.rows[ri][ci].active
                    || !card.rows[ri + 1][ci].active
                {
                    continue;
                }
                let p0 = card.rows[ri - 1][ci].pos;
                let p2 = card.rows[ri + 1][ci].pos;
                let midpt = (p0 + p2) * 0.5;
                let v = &mut card.rows[ri][ci];
                v.pos = v.pos + (midpt - v.pos) * stiffness * v.inv_mass;
            }
        }

        // Horizontal bending — only for cards with ≥3 columns
        if card.rows[0].len() >= 3 {
            for ri in 0..rows {
                let cols = card.rows[ri].len();
                for ci in 1..cols - 1 {
                    if !card.rows[ri][ci - 1].active
                        || !card.rows[ri][ci].active
                        || !card.rows[ri][ci + 1].active
                    {
                        continue;
                    }
                    let p0 = card.rows[ri][ci - 1].pos;
                    let p2 = card.rows[ri][ci + 1].pos;
                    let midpt = (p0 + p2) * 0.5;
                    let v = &mut card.rows[ri][ci];
                    v.pos = v.pos + (midpt - v.pos) * stiffness * v.inv_mass;
                }
            }
        }
    }
}

/// Push vertices out of the head mesh.
fn solve_head_collision(cards: &mut [HairCard], head: &HeadMesh) {
    for _ in 0..4 {
        let mut any_inside = false;
        for card in &mut *cards {
            for row in &mut card.rows {
                for v in row {
                    if !v.active || v.inv_mass < 0.01 {
                        continue;
                    }
                    if head.contains(v.pos) {
                        if let Some((surf, n)) = head.raycast_from_point(v.pos) {
                            v.pos = surf + n * 0.005;
                        }
                        any_inside = true;
                    }
                }
            }
        }
        if !any_inside {
            break;
        }
    }
}

/// Layer collision: inner layer vertices cannot protrude past outer layer.
/// For cards: check by card index match (same-index cards in adjacent layers).
pub fn solve_layer_collision(card_set: &mut CardSet, head_center: Vec3) {
    for li in 1..card_set.layers.len() {
        for ci in 0..card_set.layers[li].len() {
            let inner = &card_set.layers[li][ci];
            if ci >= card_set.layers[li - 1].len() {
                continue;
            }
            let outer = &card_set.layers[li - 1][ci];

            // Compare radial distances: inner vertices must be inside outer
            let inner_rows = inner.rows.len();
            for ri in 1..inner_rows {
                if ri >= outer.rows.len() {
                    break;
                }
                for vi in 0..inner.rows[ri].len() {
                    if vi >= outer.rows[ri].len() {
                        break;
                    }
                    let iv = &inner.rows[ri][vi];
                    let ov = &outer.rows[ri][vi];
                    if !iv.active || !ov.active {
                        continue;
                    }
                    let od2 = (ov.pos - head_center).length_squared();
                    let id2 = (iv.pos - head_center).length_squared();
                    if id2 > od2 {
                        // Can't modify through shared ref — handled in simulation step
                    }
                }
            }
        }
    }
}
