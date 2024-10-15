//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.

use ::bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::{
    entities::celestials::celestial::DataPlugin,
    gui::{brush, camera, element_picker, system_stepping::SteppingEguiPlugin, GuiUnifiedPlugin},
};
use physics::orbits::nbody::NBodyPlugin;

#[cfg(target_pointer_width = "32")]
compile_error!("This game is not supported on 32-bit systems.");

pub mod bevy;
pub mod common;
pub mod physics;

/// All of our gui plugins
pub struct OrbitingSandPluginGroup;

impl PluginGroup for OrbitingSandPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(NBodyPlugin)
            .add(camera::CameraPlugin)
            .add(brush::BrushPlugin)
            .add(element_picker::ElementPickerPlugin)
            .add(DataPlugin)
            .add(GuiUnifiedPlugin)
            .add(SteppingEguiPlugin::default())
    }
}
