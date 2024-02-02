//! This module contains all the physics related code.
//!
//! When contributing to this module, please keep the following things in mind:
//! * I try to maintain this module as "game engine agnostic" as possible.
//!   That way it is usable in other projects and if we ever decide to switch game engines.
//!   Please use internal types as much as possible.
//! * Physics should be highly unit tested.

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
