//! This module contains all the GUI related code.
//! Things that are drawn to via screen coordinates rather than world coordinates.

use bevy::{
    app::{Plugin, PluginGroup, PluginGroupBuilder, Startup},
    ecs::system::Commands,
};

use self::{brush::BrushPlugin, camera::CameraPlugin, system_stepping::SteppingEguiPlugin};

pub mod brush;
pub mod camera;
pub mod element_picker;
pub mod system_stepping;

/// TODO: I'm not sure what this is for. Why is it seperate from [`GuiPluginGroup`]?
pub struct GuiUnifiedPlugin;

impl Plugin for GuiUnifiedPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, Self::setup);
    }
}

impl GuiUnifiedPlugin {
    /// Runs the setup function for each gui plugin
    pub fn setup(mut commands: Commands) {
        let camera = CameraPlugin::setup_main_camera(&mut commands);
        BrushPlugin::create_brush(&mut commands, camera);
    }
}

/// All of our gui plugins
pub struct GuiPluginGroup;

impl PluginGroup for GuiPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(camera::CameraPlugin)
            .add(brush::BrushPlugin)
            .add(element_picker::ElementPickerPlugin)
            .add(GuiUnifiedPlugin)
            .add(SteppingEguiPlugin::default())
    }
}
