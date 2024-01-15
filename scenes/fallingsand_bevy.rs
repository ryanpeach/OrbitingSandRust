use bevy::{
    core_pipeline::clear_color::ClearColorConfig, log::LogPlugin, prelude::*,
    sprite::MaterialMesh2dBundle,
};
use orbiting_sand::nodes::camera::GameCamera;
use orbiting_sand::nodes::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder};

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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                GameCamera::zoom_camera_system,
                GameCamera::move_camera_system,
            ),
        )
        .add_systems(
            Update,
            (
                CelestialData::process_system,
                CelestialData::redraw_system.after(CelestialData::process_system),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    // Create a Celestial
    let planet = EarthLikeBuilder::new().build();
    let mesh: Handle<Mesh> = planet.get_combined_mesh().load_bevy_mesh(&mut meshes);
    let image: Image = planet.calc_combined_mesh_texture().to_bevy_image();
    let material: Handle<ColorMaterial> = materials.add(asset_server.add(image).into());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..Default::default()
        },
        planet,
    ));
}
