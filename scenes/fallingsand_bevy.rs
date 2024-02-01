use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::{core_pipeline::clear_color::ClearColorConfig, log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use orbiting_sand::entities::camera::CameraPlugin;
use orbiting_sand::entities::celestials::celestial::CelestialDataPlugin;
use orbiting_sand::entities::celestials::sun::SunBuilder;
use orbiting_sand::entities::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder};
use orbiting_sand::gui::brush::{BrushPlugin, BrushRadius};
use orbiting_sand::gui::camera_window::CameraWindowPlugin;
use orbiting_sand::gui::element_picker::ElementPickerPlugin;

use orbiting_sand::physics::orbits::components::{Mass, Velocity};
use orbiting_sand::physics::orbits::nbody::NBodyPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
        ))
        .add_plugins(BrushPlugin)
        .add_plugins(ElementPickerPlugin)
        .add_plugins(NBodyPlugin)
        .add_plugins(CelestialDataPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(CameraWindowPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Create a 2D camera
    let camera = commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            },
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0) * 100.0),
            ..Default::default()
        })
        .id();

    // Create the brush
    let brush = commands
        .spawn((
            BrushRadius(0.5),
            Transform::from_translation(Vec3::new(0., 0., 0.)),
        ))
        .id();

    // Parent the brush to the camera
    commands.entity(camera).push_children(&[brush]);

    // Create earth
    let planet_data = EarthLikeBuilder::new().build();
    CelestialData::setup(
        planet_data,
        Velocity(Vec2::new(0., 1200.)),
        Vec2::new(-10000., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
    );

    // Create earth2
    let planet_data = EarthLikeBuilder::new().build();
    CelestialData::setup(
        planet_data,
        Velocity(Vec2::new(0., -1200.)),
        Vec2::new(10000., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
    );

    // Create a sun
    let sun_data = SunBuilder::new().build();
    let sun_id = CelestialData::setup(
        sun_data,
        Velocity(Vec2::new(0., 0.)),
        Vec2::new(0., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
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
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(20.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
        ));
    }

    // Parent the camera to the sun
    commands.entity(sun_id).push_children(&[camera]);
}
