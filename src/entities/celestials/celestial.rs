#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::core::FrameCount;
use bevy::ecs::component::Component;

use bevy::ecs::entity::Entity;

use bevy::ecs::query::{With, Without};
use bevy::ecs::system::{Commands, Query, Res, ResMut};

use bevy::hierarchy::{BuildChildren, Parent};

use bevy::math::{Vec2, Vec3};
use bevy::prelude::SpatialBundle;
use bevy::render::mesh::Mesh;

use bevy::render::view::Visibility;
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy::time::Time;

use bevy::transform::components::Transform;

use hashbrown::HashMap;

use crate::entities::camera::OverlayLayer1;
use crate::physics::fallingsand::data::element_directory::{ElementGridDir, Textures};


use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::orbits::components::{GravitationalField, Mass, Velocity};
use crate::physics::util::clock::Clock;

/// A component that represents a chunk by its index in the directory
#[derive(Component, Debug, Clone, Copy)]
pub struct CelestialChunkIdk(ChunkIjkVector);

/// Put this alongside the mesh that represents the heat map
#[derive(Component, Debug, Clone, Copy)]
pub struct HeatMapMaterial;

/// Put this alongside the mesh that represents the falling sand itself
#[derive(Component, Debug, Clone, Copy)]
pub struct FallingSandMaterial;

/// A plugin that adds the CelestialData system
pub struct CelestialDataPlugin;

impl Plugin for CelestialDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, CelestialData::process_system);
    }
}

/// Acts as a cache for a radial mesh's meshes and textures
#[derive(Component)]
pub struct CelestialData {
    pub element_grid_dir: ElementGridDir,
}

impl CelestialData {
    /// Creates a new CelestialData
    pub fn new(mut element_grid_dir: ElementGridDir) -> Self {
        element_grid_dir.recalculate_everything();
        Self { element_grid_dir }
    }

    // /// Save the combined mesh and textures to a directory
    // /// As well as all the chunks
    // pub fn save(&self, ctx: &mut Context, dir_path: &str) -> Result<(), ggez::GameError> {
    //     let img = self.get_combined_mesh_texture().1.to_image(ctx);
    //     let combined_path = format!("{}/combined.png", dir_path);
    //     img.encode(ctx, ggez::graphics::ImageEncodingFormat::Png, combined_path)?;
    //     self.data.element_grid_dir.save(ctx, dir_path)
    // }

    // pub fn calc_combined_mesh_outline(&self) -> OwnedMeshData {
    //     OwnedMeshData::combine(
    //         &self
    //             .element_grid_dir
    //             .get_coordinate_dir()
    //             .get_mesh_data(MeshDrawMode::Outline),
    //     )
    // }
    // pub fn calc_combined_mesh_wireframe(&self) -> OwnedMeshData {
    //     OwnedMeshData::combine(
    //         &self
    //             .element_grid_dir
    //             .get_coordinate_dir()
    //             .get_mesh_data(MeshDrawMode::TriangleWireframe),
    //     )
    // }
    // /// Only recalculates the mesh for the combined mesh, not the texture
    // fn calc_combined_mesh(element_grid_dir: &ElementGridDir) -> OwnedMeshData {
    //     OwnedMeshData::combine(
    //         &element_grid_dir
    //             .get_coordinate_dir()
    //             .get_mesh_data(MeshDrawMode::TexturedMesh),
    //     )
    // }
    // /// Retrieves the combined mesh
    // pub fn get_combined_mesh(&self) -> &OwnedMeshData {
    //     &self.combined_mesh
    // }
    // /// Only recalculates the texture for the combined mesh, not the mesh itself
    // pub fn calc_combined_mesh_texture(&self) -> RawImage {
    //     RawImage::combine(self.all_textures.clone(), self.combined_mesh.uv_bounds)
    // }

    /// Something to call every frame
    /// This calculates only 1/9th of the grid each frame
    /// for maximum performance
    pub fn process(&mut self, current_time: Clock) -> HashMap<ChunkIjkVector, Textures> {
        self.element_grid_dir.process(current_time);
        self.element_grid_dir.get_updated_target_textures()
    }

    /// Something to call every frame
    /// This is the same as process, but it processes the entire grid
    pub fn process_full(&mut self, current_time: Clock) -> HashMap<ChunkIjkVector, Textures> {
        self.element_grid_dir.process_full(current_time);
        self.element_grid_dir.get_textures()
    }

    /// Retrieves the element directory
    pub fn get_element_dir(&self) -> &ElementGridDir {
        &self.element_grid_dir
    }

    /// Retrieves the element directory mutably
    pub fn get_element_dir_mut(&mut self) -> &mut ElementGridDir {
        &mut self.element_grid_dir
    }
}

/// Bevy Systems
impl CelestialData {
    /// Draws all the chunks and sets them up as child entities of the celestial
    /// TODO: Should this be a system
    #[allow(clippy::too_many_arguments)]
    pub fn setup(
        mut celestial: CelestialData,
        velocity: Velocity,
        translation: Vec2,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &Res<AssetServer>,
        gravitational: bool,
    ) -> Entity {
        // Create all the chunk meshes as pairs of ChunkIjkVector and Mesh2dBundle
        let mut children = Vec::new();
        let element_dir = celestial.get_element_dir_mut();
        let coordinate_dir = element_dir.get_coordinate_dir();
        let mut textures = element_dir.get_textures();
        for i in 0..coordinate_dir.get_num_layers() {
            for j in 0..coordinate_dir.get_layer_num_concentric_chunks(i) {
                for k in 0..coordinate_dir.get_layer_num_radial_chunks(i) {
                    let chunk_ijk = ChunkIjkVector::new(i, j, k);
                    let celestial_chunk_id = CelestialChunkIdk(chunk_ijk);
                    let mesh = coordinate_dir
                        .get_chunk_at_idx(chunk_ijk)
                        .calc_chunk_meshdata()
                        .load_bevy_mesh(meshes);

                    let textures = textures.remove(&chunk_ijk).unwrap();
                    let heat_material = textures.heat_texture.unwrap().to_bevy_image();
                    let sand_material = textures.texture.unwrap().to_bevy_image();

                    // Create the falling sand material
                    let chunk = commands
                        .spawn((
                            celestial_chunk_id,
                            MaterialMesh2dBundle {
                                mesh: mesh.into(),
                                material: materials.add(asset_server.add(sand_material).into()),
                                ..Default::default()
                            },
                            FallingSandMaterial,
                        ))
                        .id();

                    // Now create the heat map
                    // TODO: This could be optimized by just using the outline
                    let mesh = coordinate_dir
                        .get_chunk_at_idx(chunk_ijk)
                        .calc_chunk_meshdata()
                        .load_bevy_mesh(meshes);
                    let heat_chunk = commands
                        .spawn((
                            celestial_chunk_id,
                            MaterialMesh2dBundle {
                                mesh: mesh.into(),
                                material: materials.add(asset_server.add(heat_material).into()),
                                // Move the heat map to the front
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                // Turning off the heat map for now
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                            HeatMapMaterial,
                            OverlayLayer1,
                        ))
                        .id();

                    // Parent celestial to chunk
                    children.push(chunk);
                    children.push(heat_chunk);
                }
            }
        }

        // Create a Celestial
        let celestial_id = {
            if gravitational {
                commands
                    .spawn((
                        celestial.get_element_dir().get_total_mass(),
                        velocity,
                        celestial,
                        SpatialBundle::from_transform(Transform::from_translation(
                            translation.extend(0.0),
                        )),
                        GravitationalField,
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        celestial.get_element_dir().get_total_mass(),
                        velocity,
                        celestial,
                        SpatialBundle::from_transform(Transform::from_translation(
                            translation.extend(0.0),
                        )),
                    ))
                    .id()
            }
        };

        // Parent the celestial to all the chunks
        commands
            .entity(celestial_id)
            .push_children(children.as_slice());

        // Return the celestial
        celestial_id
    }

    /// Run this system every frame to update the celestial
    pub fn process_system(
        mut celestial: Query<(Entity, &mut CelestialData, &mut Mass)>,
        mut falling_sand_materials: Query<
            (&Parent, &mut Handle<ColorMaterial>, &CelestialChunkIdk),
            With<FallingSandMaterial>,
        >,
        mut heat_materials: Query<
            (&Parent, &mut Handle<ColorMaterial>, &CelestialChunkIdk),
            (With<HeatMapMaterial>, Without<FallingSandMaterial>),
        >,
        mut materials: ResMut<Assets<ColorMaterial>>,
        asset_server: Res<AssetServer>,
        time: Res<Time>,
        frame: Res<FrameCount>,
    ) {
        for (celestial_id, mut celestial, mut mass) in celestial.iter_mut() {
            let mut new_textures: HashMap<ChunkIjkVector, Textures> =
                celestial.process(Clock::new(time.as_generic(), frame.as_ref().to_owned()));
            mass.0 = celestial.get_element_dir().get_total_mass().0;
            debug_assert_ne!(mass.0, 0.0, "Celestial mass is 0");

            // Update the falling sand materials
            for (parent, material_handle, chunk_ijk) in falling_sand_materials.iter_mut() {
                if parent.get() == celestial_id && new_textures.contains_key(&chunk_ijk.0) {
                    let material = materials.get_mut(&*material_handle).unwrap();
                    let new_texture = new_textures
                        .get_mut(&chunk_ijk.0)
                        .unwrap()
                        .texture
                        .take()
                        .unwrap()
                        .to_bevy_image();
                    material.texture = Some(asset_server.add(new_texture));
                }
            }

            // Update the heat textures
            for (parent, material_handle, chunk_ijk) in heat_materials.iter_mut() {
                if parent.get() == celestial_id && new_textures.contains_key(&chunk_ijk.0) {
                    let material = materials.get_mut(&*material_handle).unwrap();
                    let new_texture = new_textures
                        .get_mut(&chunk_ijk.0)
                        .unwrap()
                        .heat_texture
                        .take()
                        .unwrap()
                        .to_bevy_image();
                    material.texture = Some(asset_server.add(new_texture));
                }
            }
        }
    }
}
