//! This module contains all the physics related code.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod fallingsand;
pub mod heat;
pub mod orbits;
pub mod util;

pub struct PhysicsPluginGroup;

impl PluginGroup for PhysicsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(orbits::nbody::NBodyPlugin)
    }
}
