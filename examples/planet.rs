//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.
use bevy::app::App;
use bevy::app::PostStartup;
use bevy::asset::AssetServer;
use bevy::asset::Assets;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::BuildChildren;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::ImagePlugin;
use bevy::prelude::Mesh;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::With;
use bevy::render::camera::ClearColor;
use bevy::sprite::ColorMaterial;
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use orbiting_sand::bevy::entities::celestials::celestial;
use orbiting_sand::bevy::entities::celestials::earthlike;
use orbiting_sand::bevy::gui::camera::MainCamera;

use bevy::prelude::PluginGroup;

use orbiting_sand::bevy::gui::camera::CelestialIdx;

use orbiting_sand::OrbitingSandPluginGroup;

/// Create the bevy app
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::DEBUG,
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=info,bevy_egui=info,naga=info"
                        .to_string(),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(low_latency_window_plugin()),
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(OrbitingSandPluginGroup)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PostStartup, planet_only_setup)
        .run();
}

/// Creates just a planet
fn planet_only_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: Query<Entity, With<MainCamera>>,
    asset_server: Res<AssetServer>,
) {
    // Create earth
    let planet_data = earthlike::Builder::new().build();
    let planet_id = celestial::Builder::new(&mut CelestialIdx(0), "Earth".to_string(), planet_data)
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Parent the camera to the sun
    commands.entity(planet_id).push_children(&[camera.single()]);
}
