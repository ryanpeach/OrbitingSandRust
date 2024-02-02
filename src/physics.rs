//! This module contains all the physics related code.

use bevy::app::{PluginGroup, PluginGroupBuilder};

use self::orbits::gravity_vis::GravityFieldPlugin;

pub mod fallingsand;
pub mod orbits;
pub mod util;

pub struct PhysicsPluginGroup;

impl PluginGroup for PhysicsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(orbits::nbody::NBodyPlugin)
            .add(GravityFieldPlugin)
    }
}
