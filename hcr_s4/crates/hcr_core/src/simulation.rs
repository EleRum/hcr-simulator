//! Top-level simulation: head + cards + clipper + cutting + debris.
//!
//! Ported from `hcr_s2\src\simulation.rs`, adapted for the HairCard model.

use crate::clipper::{ClipperState, RuntimeCommand};
use crate::config::SimConfig;
use crate::hair::{CardSet, HairCard};
use crate::head::HeadMesh;
use crate::pbd;
use crate::types::Vec3;

/// Snapshot for serialization / network transfer.
#[cfg(feature = "snapshot")]
#[derive(serde::Serialize)]
pub struct CardSnapshot {
    pub name: String,
    pub layer: u8,
    pub vertices: Vec<[f32; 3]>,
    pub rows: usize,
    pub cols: usize,
    pub active_count: usize,
}

#[cfg(feature = "snapshot")]
#[derive(serde::Serialize)]
pub struct ClipperSnapshot {
    pub pos: [f32; 3],
    pub radius: f32,
    pub is_cutting: bool,
    pub blocked: bool,
}

#[cfg(feature = "snapshot")]
#[derive(serde::Serialize)]
pub struct DebrisSnapshot {
    pub points: Vec<[f32; 3]>,
}

#[cfg(feature = "snapshot")]
#[derive(serde::Serialize)]
pub struct SimSnapshot {
    pub tick: u64,
    pub cards: Vec<CardSnapshot>,
    pub clipper: ClipperSnapshot,
    pub debris: Vec<DebrisSnapshot>,
}

pub struct Simulation {
    pub config: SimConfig,
    pub head: HeadMesh,
    pub cards: CardSet,
    pub clipper: ClipperState,
    pub debris: Vec<Vec<Vec3>>,
    pub tick: u64,
}

impl Simulation {
    pub fn new(config: SimConfig) -> Self {
        let head = HeadMesh::load_obj("assets/head.obj", config.head_center);
        let clipper = ClipperState::new(
            config.clipper.initial_pos,
            config.clipper.radius,
            config.clipper.safety_margin,
        );

        // Generate procedural cards as fallback (will be replaced by Blender cards in M2)
        let cards = crate::hair::generate_procedural_cards(&head, 4, 32);

        Self {
            config,
            head,
            cards,
            clipper,
            debris: Vec::new(),
            tick: 0,
        }
    }

    /// Replace procedural cards with loaded Blender cards.
    pub fn set_cards(&mut self, cards: CardSet) {
        self.cards = cards;
    }

    /// One physics step.
    pub fn step(&mut self) {
        let dt = self.config.dt;
        let grav = self.config.gravity;
        let damp = self.config.pbd.damping;
        let stiff = 0.1;

        // Collect all cards into a flat Vec for the solver
        let mut all_cards: Vec<HairCard> = self.cards.layers.iter()
            .flat_map(|l| l.clone())
            .collect();

        // 1. Verlet prediction
        pbd::step_verlet(&mut all_cards, grav, dt, damp);

        // 2. PBD constraints
        for _ in 0..self.config.pbd.iterations {
            pbd::solve_constraints(&mut all_cards, &self.head, stiff);
            // TODO: layer collision (inner cannot protrude outer)
        }

        // 3. Post-constraint velocity damping
        pbd::damp_velocity(&mut all_cards, self.config.pbd.post_damping);

        // 4. Cutting
        if self.clipper.is_cutting {
            self.cut_step(&mut all_cards);
        }

        // 5. Debris physics
        for frag in &mut self.debris {
            for p in frag {
                *p = *p + grav * (dt * dt);
                if p.z < self.config.world.floor_z {
                    p.z = self.config.world.floor_z;
                }
            }
        }
        self.debris.retain(|f| f.iter().any(|p| p.z > self.config.world.floor_z + 0.01));
        if self.debris.len() > 100 {
            self.debris.drain(..self.debris.len() - 100);
        }

        // Write cards back
        self.rebuild_layers(&all_cards);

        self.tick += 1;
    }

    /// Perform cutting: for each active card, find tipmost rows inside
    /// the clipper sphere and deactivate them. Outer layers shield inner.
    fn cut_step(&mut self, all_cards: &mut [HairCard]) {
        // Check safety: clipper must be at least safety_margin from head surface
        let clipper_to_center = (self.clipper.position - self.config.head_center).length();
        let head_approx_radius = 0.4; // rough head radius at clipper height
        self.clipper.blocked = (clipper_to_center - head_approx_radius) < self.clipper.safety_margin;

        if self.clipper.blocked {
            return;
        }

        let r2 = self.clipper.radius * self.clipper.radius;

        // Cut from outer layer to inner — outer shields inner
        for li in 0..self.cards.layers.len() {
            // Collect card indices to cut in this layer
            let mut layer_has_cut = false;

            for ci in 0..self.cards.layers[li].len() {
                let card = &self.cards.layers[li][ci];

                // Find tipmost row fully inside the clipper sphere
                if let Some(cut_row) = card.tipmost_row_in_sphere(self.clipper.position, r2) {
                    if cut_row >= 1 {
                        // Mark the card as having been cut
                        layer_has_cut = true;
                    }
                }
            }

            // Actually perform cuts on the mutable all_cards
            if layer_has_cut {
                // Find the global index offset for this layer
                let offset: usize = self.cards.layers.iter()
                    .take(li).map(|l| l.len()).sum();

                for ci in 0..self.cards.layers[li].len() {
                    if let Some(cut_row) = all_cards[offset + ci].tipmost_row_in_sphere(
                        self.clipper.position,
                        r2,
                    ) {
                        if cut_row >= 1 {
                            let debris = all_cards[offset + ci].cut_tail(cut_row);
                            if debris.len() >= 2 {
                                self.debris.push(debris);
                            }
                        }
                    }
                }

                // Outer layer shields inner — stop here
                break;
            }
        }
    }

    /// Rebuild self.cards.layers from flat Vec after solver modifications.
    fn rebuild_layers(&mut self, all_cards: &[HairCard]) {
        let mut offset = 0;
        for li in 0..self.cards.layers.len() {
            let count = self.cards.layers[li].len();
            if offset + count <= all_cards.len() {
                self.cards.layers[li] = all_cards[offset..offset + count].to_vec();
            }
            offset += count;
        }
    }

    pub fn apply_command(&mut self, cmd: &RuntimeCommand) {
        let speed = self.config.clipper.move_speed;
        match cmd {
            RuntimeCommand::MoveUp => self.clipper.position.z += speed,
            RuntimeCommand::MoveDown => self.clipper.position.z -= speed,
            RuntimeCommand::MoveLeft => self.clipper.position.x -= speed,
            RuntimeCommand::MoveRight => self.clipper.position.x += speed,
            RuntimeCommand::MoveForward => self.clipper.position.y += speed,
            RuntimeCommand::MoveBackward => self.clipper.position.y -= speed,
            RuntimeCommand::SetTargetXz { x, z } => {
                self.clipper.position.x = *x;
                self.clipper.position.z = *z;
            }
            RuntimeCommand::ResetClipper => {
                self.clipper.position = self.config.clipper.initial_pos;
                self.clipper.is_cutting = false;
            }
            RuntimeCommand::ToggleCutting => self.clipper.is_cutting = !self.clipper.is_cutting,
            RuntimeCommand::SetGuard(g) => {
                // Guard 0..8 → safety_margin 3mm..25mm
                self.clipper.safety_margin = *g as f32 * 0.003;
            }
            RuntimeCommand::Regenerate => {
                let config = self.config.clone();
                *self = Self::new(config);
            }
        }
    }

    #[cfg(feature = "snapshot")]
    pub fn snapshot(&self) -> SimSnapshot {
        SimSnapshot {
            tick: self.tick,
            cards: self.cards.layers.iter().flatten().map(|card| {
                let rows = card.rows.len();
                let cols = card.rows.first().map(|r| r.len()).unwrap_or(0);
                CardSnapshot {
                    name: card.name.clone(),
                    layer: card.layer,
                    vertices: card.rows.iter().flatten()
                        .map(|v| [v.pos.x, v.pos.y, v.pos.z])
                        .collect(),
                    rows,
                    cols,
                    active_count: card.active_count(),
                }
            }).collect(),
            clipper: ClipperSnapshot {
                pos: [self.clipper.position.x, self.clipper.position.y, self.clipper.position.z],
                radius: self.clipper.radius,
                is_cutting: self.clipper.is_cutting,
                blocked: self.clipper.blocked,
            },
            debris: self.debris.iter().map(|f| DebrisSnapshot {
                points: f.iter().map(|p| [p.x, p.y, p.z]).collect(),
            }).collect(),
        }
    }
}
