//! This module contains all the top level bundles.
//! These are entities that are used in the game.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod camera;
pub mod celestials;

pub struct EntitiesPluginGroup;

impl PluginGroup for EntitiesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera::CameraPlugin)
            .add(celestials::celestial::CelestialDataPlugin)
    }
}
