//! This module contains all the GUI related code.
//! Things that are drawn to via screen coordinates rather than world coordinates.

use bevy::{
    app::{Plugin, PluginGroup, PluginGroupBuilder, Startup},
    ecs::system::Commands,
};

use self::{brush::BrushPlugin, camera::CameraPlugin};

pub mod brush;
pub mod camera;
pub mod element_picker;

pub struct GuiUnifiedPlugin;

impl Plugin for GuiUnifiedPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, Self::setup);
    }
}

impl GuiUnifiedPlugin {
    pub fn setup(mut commands: Commands) {
        let camera = CameraPlugin::setup_main_camera(&mut commands);
        BrushPlugin::create_brush(&mut commands, camera);
    }
}

pub struct GuiPluginGroup;

impl PluginGroup for GuiPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera::CameraPlugin)
            .add(brush::BrushPlugin)
            .add(element_picker::ElementPickerPlugin)
            .add(GuiUnifiedPlugin)
    }
}
