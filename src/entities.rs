//! This module contains all the top level bundles.
//! These are entities that are used in the game.

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod celestials;
pub mod utils;

pub struct EntitiesPluginGroup;

impl PluginGroup for EntitiesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(celestials::celestial::CelestialDataPlugin)
    }
}
