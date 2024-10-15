//! This module contains all the physics related code.
//! Most of this code tries to be bevy-agnostic. But it's not a hard rule.

pub mod fallingsand;
pub mod orbits;

/// The number of physics frames per second.
pub const PHYSICS_FRAME_RATE: f64 = 60.0;
