use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, log::LogPlugin, prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_egui::EguiPlugin;
use orbiting_sand::entities::camera::{move_camera_system, zoom_camera_system};
use orbiting_sand::entities::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder};
use orbiting_sand::gui::brush::BrushRadius;
use orbiting_sand::gui::camera_window::camera_window_system;
use orbiting_sand::gui::element_picker::ElementSelection;

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
        .add_systems(
            Update,
            (
                CelestialData::process_system,
                // CelestialData::redraw_system.after(CelestialData::process_system),
            ),
        )
        .add_systems(Update, camera_window_system)
        .add_systems(Update, ElementSelection::element_picker_system)
        .add_systems(Update, BrushRadius::draw_brush_system)
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
            ..Default::default()
        })
        .id();

    // Create the brush
    let brush_mesh = BrushRadius(1.).calc_mesh().load_bevy_mesh(&mut meshes);
    let brush = commands
        .spawn((
            BrushRadius(1.),
            MaterialMesh2dBundle {
                mesh: brush_mesh.into(),
                material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                ..default()
            },
        ))
        .id();

    // Parent the brush to the camera
    commands.entity(camera).push_children(&[brush]);

    // Create a Celestial
    let planet_data = EarthLikeBuilder::new().build();
    let celestial_id = CelestialData::setup(
        planet_data,
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
    );

    // Parent the camera to the celestial
    commands.entity(celestial_id).push_children(&[camera]);
}
