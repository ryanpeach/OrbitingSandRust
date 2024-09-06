//! This module contains all the top level bundles.
//! These are entities that are used in the game.

use bevy::app::{PluginGroup as PGTrait, PluginGroupBuilder};

pub mod celestials;
#[expect(missing_docs)]
pub mod utils;

/// The plugin group for all entities
pub struct PluginGroup;

impl PGTrait for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(celestials::celestial::DataPlugin)
    }
}
