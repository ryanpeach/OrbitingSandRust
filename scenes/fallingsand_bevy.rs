use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use orbiting_sand::nodes::{
    celestials::{
        celestial::{self, Celestial},
        earthlike::EarthLikeBuilder,
    },
    node_trait::WorldDrawable,
};
use rayon::iter::Update;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, Celestial::process_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create a Celestial and add its meshs as children
    let planet = EarthLikeBuilder::new().build();
    let mesh = planet.data.calc_combined_mesh();
    let texture = planet
        .data
        .calc_combined_mesh_texture(&mesh)
        .load_bevy_texture(&asset_server);
    let material = materials.add(texture.into());
    commands.spawn(MaterialMesh2dBundle {
        mesh: mesh.load_bevy_mesh(&mut meshes).into(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material,
        ..Default::default()
    });
}

// fn process_celestial(
//     time: Res<Time>,
//     frame_count: Res<FrameCount>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     celestial: Query<&Celestial>,
// ) {
//     for celestial in celestial.iter() {
//         // process the celestial
//         celestial.data.process(Clock{time, frame_count});

//         // get the celestials children
//         let children = celestial.get_children();

//         // All children that are MaterialMesh2dBundle's
//         // redraw their textures
//         for child in children.iter() {
//             if let Ok(mut mesh) = child.get_bundle::<MaterialMesh2dBundle>() {
//                 let texture = child.redraw(&mut mesh);
//                 mesh.material = materials.add(texture.into());
//                 break;
//             }
//         }
//     }
// }
