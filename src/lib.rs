//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.

use ::bevy::{
    app::{App, PluginGroup, PluginGroupBuilder},
    color::Color,
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::{Level, LogPlugin},
    render::{camera::ClearColor, texture::ImagePlugin},
    DefaultPlugins,
};
use bevy::{
    entities::celestials::celestial::DataPlugin,
    gui::{brush, camera, element_picker, system_stepping::SteppingEguiPlugin, GuiUnifiedPlugin},
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use physics::orbits::nbody::NBodyPlugin;

use bevy_mod_picking::low_latency_window_plugin;

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

/// Add the plugins that all examples should add
pub fn add_common_plugins(app: &mut App) -> &mut App {
    // Determine the log plugin based on whether debug assertions are enabled
    let log_plugin = if cfg!(debug_assertions) {
        LogPlugin {
            level: Level::TRACE,
            filter:
                "wgpu=error,bevy_render=info,bevy_ecs=trace,bevy_egui=info,naga=info,winit=debug"
                    .to_string(),
            ..Default::default()
        }
    } else {
        LogPlugin {
            level: Level::INFO,
            filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
            ..Default::default()
        }
    };

    app.add_plugins((
        DefaultPlugins
            .set(log_plugin)
            .set(ImagePlugin::default_nearest())
            .set(low_latency_window_plugin()),
        FrameTimeDiagnosticsPlugin,
        EguiPlugin,
    ))
    .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
    .add_plugins(OrbitingSandPluginGroup)
    .add_plugins(WorldInspectorPlugin::new())
}
