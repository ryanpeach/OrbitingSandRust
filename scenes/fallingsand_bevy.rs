use bevy::{log::LogPlugin, prelude::*, sprite::MaterialMesh2dBundle};
use orbiting_sand::{
    nodes::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder},
    physics::util::clock::Clock,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::TRACE,
            ..Default::default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, CelestialData::process_system)
        // .add_systems(
        //     Update,
        //     CelestialData::redraw_system.after(CelestialData::process_system),
        // )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create a Celestial
    let mut planet = EarthLikeBuilder::new().build();
    let mesh: Handle<Mesh> = planet.get_combined_mesh().load_bevy_mesh(&mut meshes);
    let texture: Handle<Image> = planet
        .calc_combined_mesh_texture()
        .load_bevy_texture(&asset_server);
    let material: Handle<ColorMaterial> = materials.add(texture.into());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
            ..Default::default()
        },
        planet,
    ));
}
