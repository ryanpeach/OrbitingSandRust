//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.
#[warn(
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::panic,
    clippy::trivially_copy_pass_by_ref,
    clippy::inefficient_to_string,
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_fields_in_debug,
    clippy::redundant_clone
)]
#[deny(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
#[allow(clippy::too_many_lines)]
pub mod entities;
pub mod gui;
pub mod physics;

use bevy::app::App;
use bevy::app::PostStartup;
use bevy::asset::AssetServer;
use bevy::asset::Assets;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::math::Vec2;
use bevy::prelude::default;
use bevy::prelude::BuildChildren;
use bevy::prelude::Circle;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::ImagePlugin;
use bevy::prelude::Mesh;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Transform;
use bevy::prelude::With;
use bevy::sprite::ColorMaterial;
use bevy::DefaultPlugins;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::entities::celestials::celestial;
use crate::entities::celestials::earthlike;
use crate::entities::celestials::sun;
use bevy::log::LogPlugin;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use bevy_mod_picking::DefaultPickingPlugins;
use gui::camera::MainCamera;

use bevy::prelude::PluginGroup;

use crate::gui::camera::{BackgroundLayer1, CelestialIdx};
use crate::gui::GuiPluginGroup;
use crate::physics::orbits::components::{Mass, Velocity};

use crate::physics::PhysicsPluginGroup;

/// Create the bevy app
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::TRACE,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(low_latency_window_plugin()),
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
            DefaultPickingPlugins,
        ))
        .add_plugins(GuiPluginGroup)
        .add_plugins(PhysicsPluginGroup)
        .add_plugins(entities::PluginGroup)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PostStartup, planet_only_setup)
        .run();
}

/// Creates a solar system with a sun, earth, and a bunch of asteroids.
#[allow(dead_code)]
fn solar_system_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // This indexes our created entities
    // The "new" functions will index it for you
    let mut idx = CelestialIdx(0);

    // Create earth
    let planet_data = earthlike::Builder::new().build();
    celestial::Builder::new(&mut idx, "Earth1".to_string(), planet_data)
        .translation(Vec2::new(-10000., 0.))
        .velocity(Velocity(Vec2::new(0., 1200.)))
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Create earth2
    let planet_data = earthlike::Builder::new().build();
    celestial::Builder::new(&mut idx, "Earth2".to_string(), planet_data)
        .translation(Vec2::new(10000., 0.))
        .velocity(Velocity(Vec2::new(0., -1200.)))
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Create a sun
    let sun_data = sun::Builder::new().build();
    celestial::Builder::new(&mut idx, "Sun".to_string(), sun_data).build(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
    );

    // Create a bunch of asteroids
    const NUM_ASTEROIDS: usize = 10000;
    for i in 0..NUM_ASTEROIDS {
        // Put them in a circle around the sun
        // at radius 5000 with a tangent velocity of 600
        let angle = (i as f32 / NUM_ASTEROIDS as f32) * 2.0 * std::f32::consts::PI;
        // random radius between 5000.0 and 6000.0
        let r = 5000.0 + 1000.0 * rand::random::<f32>();
        let pos = r * Vec2::new(angle.cos(), angle.sin());
        let vel = Vec2::new(angle.sin(), -angle.cos()) * 2000.0;
        commands.spawn((
            Velocity(vel),
            Mass(1.0),
            BackgroundLayer1,
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(20.)).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(pos.extend(-1.0)),
                ..default()
            },
        ));
    }
}

/// Creates just a planet
#[allow(dead_code)]
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
