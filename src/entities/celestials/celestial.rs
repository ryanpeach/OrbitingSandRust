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
use bevy::ecs::query::{With};
use bevy::ecs::system::{Commands, Query, Res, ResMut};

use bevy::hierarchy::{BuildChildren, Parent};
use bevy::math::{Vec2};

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
use crate::gui::camera_window::CameraWindowCheckboxes;
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
                CelestialDataPlugin::change_falling_sand_visibility_system,
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

/// Bevy Systems
impl CelestialDataPlugin {
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
        celestial_idx: usize,
        gravitational: bool,
    ) -> Entity {
        // Create all the chunk meshes as pairs of ChunkIjkVector and Mesh2dBundle
        let mut children = Vec::new();
        let element_dir = celestial.get_element_dir_mut();
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
                                transform: Transform::from_translation(translation.extend(3.0)),
                                visibility: Visibility::Visible,
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
                                transform: Transform::from_translation(translation.extend(2.0)),
                                visibility: Visibility::Visible,
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
                    celestial
                        .get_element_dir()
                        .get_coordinate_dir()
                        .get_radius(),
                    celestial.get_element_dir().get_total_mass(),
                    velocity,
                    celestial,
                    CelestialIdx(celestial_idx),
                    SpatialBundle {
                        transform: Transform::from_translation(translation.extend(0.0)),
                        ..Default::default()
                    },
                ))
                .id()
        };
        if gravitational {
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
    ///
    /// > [!WARNING]
    /// > TODO: Wish I could just set the gizmo to not have to have the camera window checkboxes
    /// >       and instead just draw all visible gizmos.
    /// >       but the checkbox utility in bevy_egui does not emit an event, and linking
    /// >       the systems via one as a modifier of Visibility and this as a reader
    /// >       created a system loop
    pub fn draw_wireframe_system(
        mut gizmos: Gizmos,
        mut query: Query<
            (&GizmoDrawableTriangles, &Transform, &mut Visibility),
            With<CelestialWireframe>,
        >,
        checkboxes: Res<CameraWindowCheckboxes>,
    ) {
        for (drawable, transform, mut visibility) in query.iter_mut() {
            *visibility = if checkboxes.wireframe {
                visibility::Visibility::Visible
            } else {
                visibility::Visibility::Hidden
            };
            if *visibility == visibility::Visibility::Visible {
                drawable.draw_bevy_gizmo_triangles(&mut gizmos, transform);
            }
        }
    }
    /// Draw the outline of the celestials chunks
    ///
    /// > [!WARNING]
    /// > TODO: Wish I could just set the gizmo to not have to have the camera window checkboxes
    /// >       and instead just draw all visible gizmos.
    /// >       but the checkbox utility in bevy_egui does not emit an event, and linking
    /// >       the systems via one as a modifier of Visibility and this as a reader
    /// >       created a system loop
    pub fn draw_outline_system(
        mut gizmos: Gizmos,
        mut query: Query<(&GizmoDrawableLoop, &Transform, &mut Visibility), With<CelestialOutline>>,
        checkboxes: Res<CameraWindowCheckboxes>,
    ) {
        for (drawable, transform, mut visibility) in query.iter_mut() {
            *visibility = if checkboxes.outline {
                visibility::Visibility::Visible
            } else {
                visibility::Visibility::Hidden
            };
            if *visibility == visibility::Visibility::Visible {
                drawable.draw_bevy_gizmo_loop(&mut gizmos, transform);
            }
        }
    }

    /// Change falling sand visibility
    pub fn change_falling_sand_visibility_system(
        mut query: Query<&mut Visibility, With<FallingSandMaterial>>,
        checkboxes: Res<CameraWindowCheckboxes>,
    ) {
        for mut visibility in query.iter_mut() {
            *visibility = if checkboxes.material {
                visibility::Visibility::Visible
            } else {
                visibility::Visibility::Hidden
            };
        }
    }
}
