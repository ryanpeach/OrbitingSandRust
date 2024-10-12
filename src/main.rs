//! These documents are for game developers to understand the code, rather than for players.
//! For players, we will eventually create a mdbook describing gameplay.
//! This is the entry point for the game. It installs the plugins and contains
//! a couple of setup functions for creating different scenes.
use bevy::app::App;
use bevy::app::PostStartup;
use bevy::asset::AssetServer;
use bevy::asset::Assets;
use bevy::color::palettes::css::PURPLE;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
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
use bevy::render::camera::ClearColor;
use bevy::sprite::ColorMaterial;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::low_latency_window_plugin;
use bevy_mod_picking::DefaultPickingPlugins;
use conv::ConvAsUtil;
use orbiting_sand::bevy::entities::celestials::celestial;
use orbiting_sand::bevy::entities::celestials::earthlike;
use orbiting_sand::bevy::entities::celestials::sun;
use orbiting_sand::bevy::gui::camera::MainCamera;

use bevy::prelude::PluginGroup;

use orbiting_sand::bevy::gui::camera::{BackgroundLayer1, CelestialIdx};
use orbiting_sand::bevy::gui::GuiPluginGroup;
use orbiting_sand::common::util::uom::{Mass, Velocity};

use orbiting_sand::physics::PhysicsPluginGroup;

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
        .add_plugins(GuiPluginGroup)
        .add_plugins(PhysicsPluginGroup)
        .add_plugins(orbiting_sand::bevy::entities::PluginGroup)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(PostStartup, planet_only_setup)
        .run();
}

/// The number of asteroids in [`solar_system_setup`]
const NUM_ASTEROIDS: u32 = 10000;

/// Creates a solar system with a sun, earth, and a bunch of asteroids.
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
    for i in 0..NUM_ASTEROIDS {
        // Put them in a circle around the sun
        // at radius 5000 with a tangent velocity of 600
        let angle: f32 = ((f64::from(i) / f64::from(NUM_ASTEROIDS)) * 2.0 * std::f64::consts::PI)
            .approx()
            .expect("This doesn't matter");
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
                material: materials.add(ColorMaterial::from(Color::from(PURPLE))),
                transform: Transform::from_translation(pos.extend(-1.0)),
                ..default()
            },
        ));
    }
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
