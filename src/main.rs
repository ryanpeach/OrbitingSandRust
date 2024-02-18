//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.
pub mod entities;
pub mod gui;
pub mod physics;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::entities::celestials::celestial::CelestialBuilder;
use crate::entities::celestials::earthlike::EarthLikeBuilder;
use crate::entities::celestials::sun::SunBuilder;
use crate::entities::EntitiesPluginGroup;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use bevy_mod_picking::DefaultPickingPlugins;
use gui::camera::MainCamera;

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
        .add_plugins(EntitiesPluginGroup)
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
    let planet_data = EarthLikeBuilder::new().build();
    CelestialBuilder::new(&mut idx, "Earth1".to_string(), planet_data)
        .translation(Vec2::new(-10000., 0.))
        .velocity(Velocity(Vec2::new(0., 1200.)))
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Create earth2
    let planet_data = EarthLikeBuilder::new().build();
    CelestialBuilder::new(&mut idx, "Earth2".to_string(), planet_data)
        .translation(Vec2::new(10000., 0.))
        .velocity(Velocity(Vec2::new(0., -1200.)))
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Create a sun
    let sun_data = SunBuilder::new().build();
    CelestialBuilder::new(&mut idx, "Sun".to_string(), sun_data).build(
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
                mesh: meshes.add(shape::Circle::new(20.).into()).into(),
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
    let planet_data = EarthLikeBuilder::new().build();
    let planet_id = CelestialBuilder::new(&mut CelestialIdx(0), "Earth".to_string(), planet_data)
        .build(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Parent the camera to the sun
    commands.entity(planet_id).push_children(&[camera.single()]);
}
