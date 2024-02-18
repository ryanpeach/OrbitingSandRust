use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::core::FrameCount;
use bevy::ecs::component::Component;

use bevy::ecs::entity::Entity;

use bevy::gizmos::gizmos::Gizmos;

use bevy::render::color::Color;
use bevy::render::view::{visibility, Visibility};
use bevy_mod_picking::prelude::*;

// use bevy_mod_picking::PickableBundle;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut};

use bevy::hierarchy::{BuildChildren, Parent};
use bevy::math::Vec2;

use bevy::prelude::SpatialBundle;
use bevy::render::mesh::Mesh;

use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::events::Pointer;
use bevy_mod_picking::PickableBundle;

use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy::time::{Fixed, Time};

use bevy::transform::components::Transform;

use hashbrown::HashMap;

use crate::gui::camera::{CelestialIdx, OverlayLayer2, OverlayLayer3, SelectCelestial};
use crate::physics::fallingsand::data::element_directory::{ElementGridDir, Textures};

use crate::physics::fallingsand::util::mesh::{GizmoDrawableLoop, GizmoDrawableTriangles};
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::orbits::components::{GravitationalField, Mass, Velocity};
use crate::physics::util::clock::Clock;
use crate::physics::PHYSICS_FRAME_RATE;

/// Identifies the mesh which draws the celestials chunk outlines
#[derive(Component)]
pub struct CelestialOutline;

/// Identifies the mesh which draws the celestial cell wireframes
#[derive(Component)]
pub struct CelestialWireframe;

/// A component that represents a chunk by its index in the directory
#[derive(Component, Debug, Clone, Copy)]
pub struct CelestialChunkIdk(ChunkIjkVector);

/// Put this alongside the mesh that represents the falling sand itself
#[derive(Component, Debug, Clone, Copy)]
pub struct FallingSandMaterial;

/// A plugin that adds the CelestialData system
pub struct CelestialDataPlugin;

impl Plugin for CelestialDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, Self::process_system);
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / PHYSICS_FRAME_RATE));
        app.add_systems(
            Update,
            (
                CelestialDataPlugin::draw_wireframe_system,
                CelestialDataPlugin::draw_outline_system,
            ),
        );
        app.add_event::<SelectCelestial>();
    }
}

/// Acts as a cache for a polar mesh's meshes and textures
#[derive(Component)]
pub struct CelestialData {
    /// The elements in this celestial
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

/// Create a celestial using a builder pattern
pub struct CelestialBuilder {
    name: String,
    celestial_data: CelestialData,
    velocity: Velocity,
    translation: Vec2,
    celestial_idx: CelestialIdx,
    gravitational: bool,
    children: Vec<Entity>,
}

impl CelestialBuilder {
    /// Create a new celestial builder
    pub fn new(idx: &mut CelestialIdx, name: String, data: CelestialData) -> Self {
        let out = Self {
            name: name,
            celestial_data: data,
            celestial_idx: idx.clone(),
            velocity: Velocity(Vec2::new(0., 0.)),
            translation: Vec2::new(0., 0.),
            gravitational: true,
            children: Vec::new(),
        };
        *idx = *idx + 1;
        out
    }

    /// Set the velocity of the celestial
    pub fn velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set the translation of the celestial
    pub fn translation(mut self, translation: Vec2) -> Self {
        self.translation = translation;
        self
    }

    /// Set the gravitational field of the celestial
    pub fn gravitational(mut self, gravitational: bool) -> Self {
        self.gravitational = gravitational;
        self
    }

    /// Build the celestial
    pub fn build(
        self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        // Create all the chunk meshes as pairs of ChunkIjkVector and Mesh2dBundle
        let mut children = Vec::new();
        let element_dir = self.celestial_data.get_element_dir();
        let coordinate_dir = element_dir.get_coordinate_dir();
        let mut textures = element_dir.get_textures();
        for i in 0..coordinate_dir.get_num_layers() {
            for j in 0..coordinate_dir.get_layer_num_concentric_chunks(i) {
                for k in 0..coordinate_dir.get_layer_num_tangential_chunkss(i) {
                    let chunk_ijk = ChunkIjkVector::new(i, j, k);
                    let celestial_chunk_id = CelestialChunkIdk(chunk_ijk);
                    let mesh = coordinate_dir
                        .get_chunk_at_idx(chunk_ijk)
                        .calc_chunk_meshdata();
                    let mesh_handle = mesh.load_bevy_mesh(meshes);
                    let wireframe = coordinate_dir
                        .get_chunk_at_idx(chunk_ijk)
                        .calc_chunk_triangle_wireframe();
                    let outline = coordinate_dir
                        .get_chunk_at_idx(chunk_ijk)
                        .calc_chunk_outline();

                    let textures = textures.remove(&chunk_ijk).unwrap();
                    let sand_material = textures.texture.unwrap().to_bevy_image();

                    // Create the falling sand material
                    let chunk = commands
                        .spawn((
                            celestial_chunk_id,
                            MaterialMesh2dBundle {
                                mesh: mesh_handle.into(),
                                material: materials.add(asset_server.add(sand_material).into()),
                                visibility: Visibility::Visible,
                                ..Default::default()
                            },
                            // mesh.calc_bounds(),
                            PickableBundle::default(), // Makes the entity pickable
                            FallingSandMaterial,
                        ))
                        .id();

                    // Now create the gizmos
                    let outline_entity = commands
                        .spawn((
                            GizmoDrawableLoop::new(outline, Color::RED),
                            SpatialBundle {
                                transform: Transform::from_translation(
                                    self.translation.extend(3.0),
                                ),
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                            CelestialOutline,
                            OverlayLayer3,
                        ))
                        .id();
                    let wireframe_entity = commands
                        .spawn((
                            GizmoDrawableTriangles::new(wireframe, Color::WHITE),
                            SpatialBundle {
                                transform: Transform::from_translation(
                                    self.translation.extend(2.0),
                                ),
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                            CelestialWireframe,
                            OverlayLayer2,
                        ))
                        .id();

                    // Parent celestial to chunk
                    children.push(chunk);
                    children.push(outline_entity);
                    children.push(wireframe_entity);
                }
            }
        }

        // Create a Celestial
        let celestial_id = {
            commands
                .spawn((
                    // Physics
                    self.celestial_data
                        .get_element_dir()
                        .get_coordinate_dir()
                        .get_radius(),
                    self.celestial_data.get_element_dir().get_total_mass(),
                    self.velocity,
                    self.celestial_data,
                    self.celestial_idx,
                    SpatialBundle {
                        transform: Transform::from_translation(self.translation.extend(0.0)),
                        ..Default::default()
                    },
                ))
                .id()
        };
        if self.gravitational {
            commands.entity(celestial_id).insert(GravitationalField);
        }

        // Parent the celestial to all the chunks
        commands
            .entity(celestial_id)
            .push_children(children.as_slice());

        // And create events
        commands
            .entity(celestial_id)
            .insert(On::<Pointer<Down>>::send_event::<SelectCelestial>());

        // Return the celestial
        celestial_id
    }
}

/// Bevy Systems
impl CelestialDataPlugin {
    /// Run this system every frame to update the celestial
    #[allow(clippy::type_complexity)]
    pub fn process_system(
        mut celestial: Query<(Entity, &mut CelestialData, &mut Mass)>,
        mut falling_sand_materials: Query<
            (&Parent, &mut Handle<ColorMaterial>, &CelestialChunkIdk),
            With<FallingSandMaterial>,
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
        }
    }
    /// Draw the wireframe of the celestials cells
    pub fn draw_wireframe_system(
        mut gizmos: Gizmos,
        query: Query<(&GizmoDrawableTriangles, &Transform, &Visibility), With<CelestialWireframe>>,
    ) {
        for (drawable, transform, visibility) in query.iter() {
            if *visibility == visibility::Visibility::Visible {
                drawable.draw_bevy_gizmo_triangles(&mut gizmos, transform);
            }
        }
    }
    /// Draw the outline of the celestials chunks
    pub fn draw_outline_system(
        mut gizmos: Gizmos,
        query: Query<(&GizmoDrawableLoop, &Transform, &Visibility), With<CelestialOutline>>,
    ) {
        for (drawable, transform, visibility) in query.iter() {
            if *visibility == visibility::Visibility::Visible {
                drawable.draw_bevy_gizmo_loop(&mut gizmos, transform);
            }
        }
    }
}
