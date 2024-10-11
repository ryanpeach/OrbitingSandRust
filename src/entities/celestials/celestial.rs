//! Celestial entities and their data
//! A celestial is a large body in space, such as a planet or star
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::asset::{AssetEvent, AssetId, AssetServer, Assets, Handle};
use bevy::color::Srgba;
use bevy::core::{FrameCount, Name};
use bevy::ecs::component::Component;

use bevy::color::palettes::css::RED;
use bevy::ecs::entity::Entity;
use bevy::gizmos::gizmos::Gizmos;

use bevy::ecs::event::EventReader;
use bevy::prelude::Resource;
use bevy::render::texture::Image;
use bevy::render::view::{ViewVisibility, Visibility, VisibilityBundle};
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

use bevy::transform::components::{GlobalTransform, Transform};

use hashbrown::HashMap;

use crate::gui::camera::{CelestialIdx, OverlayLayer2, OverlayLayer3, SelectCelestial};
use crate::physics::fallingsand::data::element_directory::{ElementGridDir, Textures};

use crate::physics::fallingsand::mesh::chunk_coords::{VertexMode, VertexSettings};
use crate::physics::fallingsand::util::mesh::{GizmoDrawableGrid, GizmoDrawableLoop};
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::orbits::components::{GravitationalField, Mass, Velocity};
use crate::physics::util::clock::Clock;
use crate::physics::PHYSICS_FRAME_RATE;

/// Identifies the mesh which draws the celestials chunk outlines
#[derive(Component)]
pub struct Outline;

/// Identifies the mesh which draws the celestial cell wireframes
#[derive(Component)]
pub struct Wireframe;

/// A component that represents a chunk by its index in the directory
#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkIjk(ChunkIjkVector);

/// Put this alongside the mesh that represents the falling sand itself
#[derive(Component, Debug, Clone, Copy)]
pub struct FallingSandMaterial;

/// A plugin that adds the `CelestialData` system
pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (Self::process_system, DataPlugin::draw_materials_system),
        );

        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / PHYSICS_FRAME_RATE));
        app.insert_resource(UpdatedTextures::default());
        app.add_systems(
            Update,
            (
                DataPlugin::draw_wireframe_system,
                DataPlugin::draw_outline_system,
            ),
        );
        app.add_event::<SelectCelestial>();
    }
}

/// Acts as a cache for a polar mesh's meshes and textures
#[derive(Component)]
pub struct Data {
    /// The elements in this celestial
    pub element_grid_dir: ElementGridDir,
}

impl Data {
    /// Creates a new `CelestialData`
    #[must_use]
    pub fn new(mut element_grid_dir: ElementGridDir) -> Self {
        element_grid_dir.recalculate_everything();
        Self { element_grid_dir }
    }

    /// Something to call every frame
    /// This calculates only 1/9th of the grid each frame
    /// for maximum performance
    pub fn process(&mut self, current_time: Clock) -> HashMap<ChunkIjkVector, Textures> {
        self.element_grid_dir.process(current_time);
        self.element_grid_dir.updated_target_textures()
    }

    /// Something to call every frame
    /// This is the same as process, but it processes the entire grid
    pub fn process_full(&mut self, current_time: Clock) -> HashMap<ChunkIjkVector, Textures> {
        self.element_grid_dir.process_full(current_time);
        self.element_grid_dir.textures()
    }

    /// Retrieves the element directory
    #[must_use]
    pub fn element_dir(&self) -> &ElementGridDir {
        &self.element_grid_dir
    }

    /// Retrieves the element directory mutably
    pub fn element_dir_mut(&mut self) -> &mut ElementGridDir {
        &mut self.element_grid_dir
    }
}

/// Create a celestial using a builder pattern
pub struct Builder {
    /// The name of the celestial
    name: String,
    /// A component that wraps the element directory
    celestial_data: Data,
    /// The starting velocity of the celestial
    velocity: Velocity,
    /// The starting position of the celestial
    translation: Vec2,
    /// The index of the celestial (0 to n), used for camera control
    celestial_idx: CelestialIdx,
    /// Whether the celestial has a gravitational field
    gravitational: bool,
}

impl Builder {
    /// Create a new [`Builder`]
    pub fn new(idx: &mut CelestialIdx, name: String, data: Data) -> Self {
        let out = Self {
            name,
            celestial_data: data,
            celestial_idx: *idx,
            velocity: Velocity(Vec2::new(0., 0.)),
            translation: Vec2::new(0., 0.),
            gravitational: true,
        };
        *idx = *idx + 1;
        out
    }

    /// Set [`Builder::velocity`]
    #[must_use]
    pub fn velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set [`Builder::translation`]
    #[must_use]
    pub fn translation(mut self, translation: Vec2) -> Self {
        self.translation = translation;
        self
    }

    /// Set [`Builder::gravitational`]
    #[must_use]
    pub fn gravitational(mut self, gravitational: bool) -> Self {
        self.gravitational = gravitational;
        self
    }

    /// Build [`Entity`]
    ///
    /// # Panics
    ///
    /// Should never panic, but uses expect to handle the case where a texture is missing.
    pub fn build(
        self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        // Create all the chunk meshes as pairs of ChunkIjkVector and Mesh2dBundle
        let mut chunks = Vec::new();
        let mut wireframes = Vec::new();
        let mut outlines = Vec::new();
        let element_dir = self.celestial_data.element_dir();
        let coordinate_dir = element_dir.coordinate_dir();
        let mut textures = element_dir.textures();
        for i in 0..coordinate_dir.num_layers() {
            for j in 0..coordinate_dir.layer_num_concentric_chunks(i) {
                for k in 0..coordinate_dir.layer_num_tangential_chunkss(i) {
                    let chunk_ijk = ChunkIjkVector::new(i, j, k);
                    let celestial_chunk_id = ChunkIjk(chunk_ijk);
                    let mesh = coordinate_dir
                        .chunk_at_idx(chunk_ijk)
                        .chunk_meshdata(VertexSettings::default());
                    let mesh_handle = mesh.load_bevy_mesh(meshes);

                    // Wireframes start to look weird unless you are at a certain level of detail at a certain chunk
                    let lod = if i > 1 {
                        2
                    } else if i > 2 {
                        4
                    } else {
                        1
                    };
                    let wireframe = coordinate_dir
                        .chunk_at_idx(chunk_ijk)
                        .chunk_triangle_wireframe(VertexSettings {
                            lod,
                            mode: VertexMode::Grid,
                        });
                    let outline = coordinate_dir.chunk_at_idx(chunk_ijk).chunk_outline();

                    let textures = textures
                        .remove(&chunk_ijk)
                        .expect("There should always be a texture");
                    let sand_material = textures
                        .texture
                        .expect("There should always be a texture")
                        .to_bevy_image();

                    // Create the falling sand material
                    let chunk = commands
                        .spawn((
                            Name::new(format!("Chunk {chunk_ijk:?}")),
                            celestial_chunk_id,
                            MaterialMesh2dBundle {
                                mesh: mesh_handle.into(),
                                material: materials.add(asset_server.add(sand_material)),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            // mesh.calc_bounds(),
                            PickableBundle::default(), // Makes the entity pickable
                            FallingSandMaterial,
                        ))
                        .id();

                    // Now create the gizmos
                    let wireframe_entity = commands
                        .spawn((
                            Name::new(format!("Wireframe {chunk_ijk:?}")),
                            GizmoDrawableGrid::new(
                                wireframe,
                                Srgba {
                                    red: 0.1,
                                    green: 0.1,
                                    blue: 0.1,
                                    alpha: 0.1,
                                }
                                .into(),
                            ),
                            SpatialBundle {
                                transform: Transform::from_translation(
                                    self.translation.extend(2.0),
                                ),
                                visibility: Visibility::Visible,
                                ..Default::default()
                            },
                            Wireframe,
                            OverlayLayer2,
                        ))
                        .id();
                    let outline_entity = commands
                        .spawn((
                            Name::new(format!("Outline {chunk_ijk:?}")),
                            GizmoDrawableLoop::new(outline, RED.into()),
                            SpatialBundle {
                                transform: Transform::from_translation(
                                    self.translation.extend(3.0),
                                ),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            Outline,
                            OverlayLayer3,
                        ))
                        .id();

                    // Parent celestial to chunk
                    chunks.push(chunk);
                    wireframes.push(wireframe_entity);
                    outlines.push(outline_entity);
                }
            }
        }

        // Create a Celestial
        let celestial_id = {
            commands
                .spawn((
                    // Physics
                    Name::new(self.name.clone()),
                    self.celestial_data.element_dir().coordinate_dir().radius(),
                    self.celestial_data.element_dir().total_mass(),
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

        // Create a wireframes entity parented to the celestial
        // which itself is the parent to all the other wireframe entities
        // this enables you to change their visibility easier with egui inspector
        // and cleans up the hierarchy
        let wireframe_id = commands
            .spawn((
                Name::new("Wireframes"),
                VisibilityBundle {
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                GlobalTransform::default(),
            ))
            .id();
        commands.entity(wireframe_id).push_children(&wireframes);
        commands.entity(celestial_id).push_children(&[wireframe_id]);

        // Create an outlines entity parented to the celestial
        // which itself is the parent to all the other outline entities
        // this enables you to change their visibility easier with egui inspector
        // and cleans up the hierarchy
        let outline = commands
            .spawn((
                Name::new("Outlines"),
                VisibilityBundle {
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                GlobalTransform::default(),
            ))
            .id();
        commands.entity(outline).push_children(&outlines);
        commands.entity(celestial_id).push_children(&[outline]);

        // And create events
        commands
            .entity(celestial_id)
            .insert(On::<Pointer<Down>>::send_event::<SelectCelestial>());

        // Parent the chunks to the celestial
        commands.entity(celestial_id).push_children(&chunks);

        // Return the celestial
        celestial_id
    }
}

/// The entire list of textures created after each call to [`DataPlugin::process_system`]
#[derive(Resource, Debug, Default)]
pub struct UpdatedTextures(HashMap<AssetId<Image>, (Entity, Handle<Image>)>);

/// Bevy Systems
impl DataPlugin {
    /// Run this system every frame to update the celestial
    /// # Panics
    /// Should never panic, but uses expect to handle the case where a texture or material is missing.
    pub fn process_system(
        mut commands: Commands,
        mut celestial_query: Query<(Entity, &mut Data, &mut Mass)>,
        falling_sand_materials: Query<(Entity, &Parent, &ChunkIjk)>,
        asset_server: Res<AssetServer>,
        time: Res<Time>,
        frame: Res<FrameCount>,
    ) {
        // Process each celestial
        let mut new_textures_by_celestial: HashMap<Entity, HashMap<ChunkIjkVector, Textures>> =
            HashMap::new();
        for (celestial_id, mut celestial, mut mass) in &mut celestial_query {
            new_textures_by_celestial.insert(
                celestial_id,
                celestial.process(Clock::new(time.as_generic(), frame.as_ref().to_owned())),
            );

            // Update the mass
            mass.0 = celestial.element_dir().total_mass().0;
        }

        // Process each chunk
        let mut asset_updates: HashMap<AssetId<Image>, (Entity, Handle<Image>)> = HashMap::new();
        for (chunk_id, celestial_id, chunk_ijk) in &falling_sand_materials {
            if let Some(textures_by_chunk_ijk) =
                new_textures_by_celestial.get_mut(&celestial_id.get())
            {
                if let Some(mut texture) = textures_by_chunk_ijk.remove(&chunk_ijk.0) {
                    if let Some(bevy_image) = texture.texture.take() {
                        let handle = asset_server.add(bevy_image.to_bevy_image());
                        asset_updates.insert(handle.id(), (chunk_id, handle));
                    }
                }
            }
        }
        commands.insert_resource(UpdatedTextures(asset_updates));
    }

    /// Actually draw the updated textures on the materials after they are loaded by the asset
    /// server.
    /// WARNING: When developing, make sure this is always O(1) for each event. I don't know how
    /// many times its called per update. Do all events come in at once or one in each call?
    /// Preprocessing could happen many times.
    pub fn draw_materials_system(
        falling_sand_materials: Query<&Handle<ColorMaterial>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut asset_events: EventReader<AssetEvent<Image>>,
        mut asset_updates: ResMut<UpdatedTextures>,
    ) {
        // Process asset events
        for event in asset_events.read() {
            if let AssetEvent::<Image>::LoadedWithDependencies { id } = event {
                if let Some((chunk_id, image_handle)) = asset_updates.0.remove(id) {
                    if let Ok(material_handle) = falling_sand_materials.get(chunk_id) {
                        if let Some(material) = materials.get_mut(material_handle) {
                            material.texture = Some(image_handle);
                        }
                    }
                }
            }
        }
    }

    /// Draw the wireframe of the celestials cells
    pub fn draw_wireframe_system(
        mut gizmos: Gizmos,
        query: Query<(&GizmoDrawableGrid, &Transform, &ViewVisibility), With<Wireframe>>,
    ) {
        for (drawable, transform, visibility) in query.iter() {
            if visibility.get() {
                drawable.draw_bevy_gizmo_grid(&mut gizmos, transform);
            }
        }
    }
    /// Draw the outline of the celestials chunks
    pub fn draw_outline_system(
        mut gizmos: Gizmos,
        query: Query<(&GizmoDrawableLoop, &Transform, &ViewVisibility), With<Outline>>,
    ) {
        for (drawable, transform, visibility) in query.iter() {
            if visibility.get() {
                drawable.draw_bevy_gizmo_loop(&mut gizmos, transform);
            }
        }
    }
}
