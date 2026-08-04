//! hcr_core — Hair cutting simulation kernel.
//!
//! Pure Rust, no I/O, no Bevy, no hotaru.
//! Deterministic: fixed dt, seeded RNG, no clock reads.
//!
//! S4: Uses Blender-authored hair cards (independent quad strips)
//! with Verlet + PBD physics, and card-level cutting semantics.

pub mod types;
pub mod config;
pub mod head;
pub mod hair;
pub mod pbd;
pub mod clipper;
pub mod simulation;

#[cfg(feature = "snapshot")]
pub mod snapshot;
