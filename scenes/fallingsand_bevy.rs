use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::{core_pipeline::clear_color::ClearColorConfig, log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use orbiting_sand::entities::camera::{move_camera_system, zoom_camera_system};
use orbiting_sand::entities::celestials::sun::SunBuilder;
use orbiting_sand::entities::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder};
use orbiting_sand::gui::brush::BrushRadius;
use orbiting_sand::gui::camera_window::camera_window_system;
use orbiting_sand::gui::element_picker::ElementSelection;
use orbiting_sand::physics::orbits::components::Velocity;
use orbiting_sand::physics::orbits::nbody::leapfrog_integration_system;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::TRACE,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .insert_resource(ElementSelection::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera_system, move_camera_system))
        .add_systems(Update, CelestialData::process_system)
        .add_systems(Update, camera_window_system)
        .add_systems(Update, ElementSelection::element_picker_system)
        .add_systems(
            Update,
            (
                BrushRadius::move_brush_system,
                BrushRadius::draw_brush_system,
                BrushRadius::resize_brush_system,
                BrushRadius::apply_brush_system,
            ),
        )
        .add_systems(
            Update,
            leapfrog_integration_system.after(CelestialData::process_system),
        )
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
        Velocity(Vec2::new(0., 100.)),
        Vec2::new(-100., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
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
    );

    // Parent the camera to the sun
    commands.entity(sun_id).push_children(&[camera]);
}
