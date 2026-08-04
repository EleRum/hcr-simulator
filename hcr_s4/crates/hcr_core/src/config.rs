//! Simulation configuration.

use crate::types::Vec3;

#[derive(Clone, Debug)]
pub struct SimConfig {
    /// Fixed timestep (seconds).
    pub dt: f32,
    /// Gravity vector (typically [0, 0, -9.8] in Z-up).
    pub gravity: Vec3,
    /// Head center for collision reference.
    pub head_center: Vec3,
    /// Head approximate radii for clipper safety check.
    pub head_radii: Vec3,
    /// PBD-specific settings.
    pub pbd: PbdConfig,
    /// Clipper settings.
    pub clipper: ClipperConfig,
    /// World bounds.
    pub world: WorldConfig,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 120.0,
            gravity: Vec3::new(0.0, 0.0, -9.8),
            head_center: Vec3::ZERO,
            head_radii: Vec3::new(0.42, 0.45, 0.55),
            pbd: PbdConfig::default(),
            clipper: ClipperConfig::default(),
            world: WorldConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PbdConfig {
    /// Number of constraint solver iterations per step.
    pub iterations: u32,
    /// Verlet velocity damping (0–1, higher = more damped).
    pub damping: f32,
    /// Post-constraint velocity damping (0–1).
    pub post_damping: f32,
}

impl Default for PbdConfig {
    fn default() -> Self {
        Self { iterations: 4, damping: 0.98, post_damping: 0.95 }
    }
}

#[derive(Clone, Debug)]
pub struct ClipperConfig {
    pub initial_pos: Vec3,
    pub radius: f32,
    /// Minimum distance from head surface (guard size mapping).
    pub safety_margin: f32,
    /// Movement speed (units per command).
    pub move_speed: f32,
}

impl Default for ClipperConfig {
    fn default() -> Self {
        Self {
            initial_pos: Vec3::new(0.0, 0.7, 0.5),
            radius: 0.08,
            safety_margin: 0.003, // ~3mm minimum
            move_speed: 0.02,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldConfig {
    pub floor_z: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { floor_z: -1.5 }
    }
}
