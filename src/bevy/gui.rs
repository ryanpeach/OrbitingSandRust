//! This module contains all the GUI related code.
//! Things that are drawn to via screen coordinates rather than world coordinates.

use bevy::{
    app::{Plugin, Startup},
    asset::Assets,
    ecs::system::Commands,
    prelude::ResMut,
    render::mesh::Mesh,
    sprite::ColorMaterial,
};

use self::{brush::BrushPlugin, camera::CameraPlugin};

pub mod brush;
pub mod camera;
pub mod element_picker;
pub mod system_stepping;

pub struct GuiUnifiedPlugin;

impl Plugin for GuiUnifiedPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, Self::setup);
    }
}

impl GuiUnifiedPlugin {
    /// Runs the setup function for each gui plugin
    pub fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        CameraPlugin::setup_main_camera(&mut commands);
        BrushPlugin::create_brush(&mut commands, &mut meshes, &mut materials);
    }
}
