//! This module contains all the physics related code.
//! Most of this code tries to be bevy-agnostic. But it's not a hard rule.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod fallingsand;
pub mod orbits;

/// The number of physics frames per second.
pub const PHYSICS_FRAME_RATE: f64 = 60.0;

/// The plugin group for all physics related code.
pub struct PhysicsPluginGroup;

impl PluginGroup for PhysicsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(orbits::nbody::NBodyPlugin)
    }
}
