//! This module contains all the physics related code.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod fallingsand;
pub mod heat;
pub mod orbits;
pub mod util;

/// The number of physics frames per second.
pub const PHYSICS_FRAME_RATE: f64 = 30.0;

pub struct PhysicsPluginGroup;

impl PluginGroup for PhysicsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(orbits::nbody::NBodyPlugin)
    }
}
