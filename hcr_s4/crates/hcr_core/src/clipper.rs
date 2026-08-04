//! Clipper state + runtime commands.

use crate::types::Vec3;

/// The virtual hair clipper.
#[derive(Clone, Debug)]
pub struct ClipperState {
    pub position: Vec3,
    pub radius: f32,
    /// Minimum allowed distance from clipper edge to head surface.
    /// Maps from guard number: 0 → 3mm, 8 → 25mm.
    pub safety_margin: f32,
    pub is_cutting: bool,
    /// True when clipper is too close to the head (safety lock engaged).
    pub blocked: bool,
}

impl ClipperState {
    pub fn new(position: Vec3, radius: f32, safety_margin: f32) -> Self {
        Self {
            position,
            radius,
            safety_margin,
            is_cutting: false,
            blocked: false,
        }
    }
}

/// Commands that can be sent to the simulation from external input
/// (Blockly scripts, keyboard, MQTT).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RuntimeCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveForward,
    MoveBackward,
    SetTargetXz { x: f32, z: f32 },
    ResetClipper,
    ToggleCutting,
    /// Set guard size (0–8), maps to safety_margin 3mm–25mm.
    SetGuard(u8),
    /// Regenerate hair (for reset).
    Regenerate,
}
