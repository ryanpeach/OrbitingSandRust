//! Celestial entities and their data
//! A celestial is a large body in space, such as a planet or star
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::app::{App, FixedPostUpdate, FixedUpdate, Plugin, Update};
use bevy::asset::{AssetEvent, AssetId, AssetServer, Assets, Handle};
use bevy::color::{Color, Srgba};
use bevy::core::{FrameCount, Name};
use bevy::ecs::component::Component;

use bevy::color::palettes::css::RED;
use bevy::ecs::entity::Entity;
use bevy::gizmos::gizmos::Gizmos;

use bevy::ecs::event::EventReader;
use bevy::log::{debug, trace_once};
use bevy::prelude::Resource;
use bevy::render::texture::Image;
use bevy::render::view::{InheritedVisibility, ViewVisibility, Visibility};
use bevy_mod_picking::prelude::*;

// use bevy_mod_picking::PickableBundle;
use bevy::ecs::query::{With, Without};
use bevy::ecs::system::{Commands, Query, Res, ResMut};

use bevy::hierarchy::{BuildChildren, Parent};
use bevy::math::Vec2;

use bevy::prelude::SpatialBundle;
use bevy::render::mesh::Mesh;

use bevy_eventlistener::event_listener::On;
use bevy_mod_picking::events::Pointer;
use bevy_mod_picking::PickableBundle;

use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy::time::Time;

use bevy::transform::components::{GlobalTransform, Transform};

use crate::bevy::components::mesh::{GizmoDrawableGrid, GizmoDrawableLoop};
use crate::bevy::gui::camera::{
    CelestialIdx, MainCamera, OverlayLayer2, OverlayLayer3, SelectCelestial,
};
use crate::bevy::systemsets::{ComputeSet, DrawSet};
use crate::physics::fallingsand::data::element_directory::ElementGridDir;
use bevy::prelude::IntoSystemConfigs;
use hashbrown::HashMap;
use macros::call_log_once;

use crate::common::util::clock::Clock;
use crate::common::util::uom::{Mass, Velocity};
use crate::common::util::vectors::ChunkIjkVector;
use crate::physics::fallingsand::mesh::chunk_coords::{VertexMode, VertexSettings};
use crate::physics::orbits::nbody::GravitationalField;

/// Identifies the mesh which draws the celestials chunk outlines
#[derive(Component)]
pub struct Outline;

/// Identifies the mesh which draws the celestial cell wireframes
#[derive(Component)]
pub struct Grid;

/// Identifies a chunk
#[derive(Component)]
pub struct Chunk;

/// The bevy-egui container which groups all [`Outline`] components.
#[derive(Component)]
pub struct OutlineGroup;

/// The bevy-egui container which groups all [`Grid`] components.
#[derive(Component)]
pub struct GridGroup;

/// A component that represents a chunk by its index in the directory
#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkIjkComponent(ChunkIjkVector);

/// The bevy-egui container which groups all [`ChunkIjkComponent`] components.
#[derive(Component)]
pub struct ChunkGroup;

/// A plugin that adds the `CelestialData` system
pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        // WARNING: you cant put this `.after` anything since it is event driven.
        // That's why its not in the [`DrawSet`]
        app.add_systems(FixedUpdate, DataPlugin::draw_materials_system);
        app.add_systems(
            FixedUpdate,
            ((DataPlugin::process_system,).in_set(ComputeSet),),
        );
        app.add_systems(
            FixedPostUpdate,
            (
                DataPlugin::draw_wireframe_system,
                DataPlugin::draw_outline_system,
                DataPlugin::queue_materials_system,
            )
                .in_set(DrawSet),
        );
        // NOTE: Enable this to automatically show wireframes when you focus on a planet
        app.add_systems(
            Update,
            (
                DataPlugin::wireframe_visibility_system,
                DataPlugin::outline_visibility_system,
            ),
        );
        app.insert_resource(UpdatedTextures::default());
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
    pub fn process(&mut self, current_time: Clock) {
        self.element_grid_dir.process(current_time);
    }

    /// Something to call every frame
    /// This is the same as process, but it processes the entire grid
    pub fn process_full(&mut self, current_time: Clock) {
        self.element_grid_dir.process_full(current_time);
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
                    let celestial_chunk_id = ChunkIjkComponent(chunk_ijk);
                    let mesh = coordinate_dir
                        .chunk_at_idx(chunk_ijk)
                        .chunk_meshdata(VertexSettings::default());

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
                                mesh: mesh.load_bevy_mesh(meshes).into(),
                                material: materials.add(asset_server.add(sand_material)),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            // mesh.calc_bounds(),
                            PickableBundle::default(), // Makes the entity pickable
                            Chunk,
                        ))
                        .id();

                    // Now create the gizmos
                    let wireframe_entity = commands
                        .spawn((
                            Name::new(format!("Cell Grid {chunk_ijk:?}")),
                            celestial_chunk_id,
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
                            // This will enable frustum culling via ViewVisibility by making a
                            // transparent mesh
                            // which will enforce bounds
                            MaterialMesh2dBundle {
                                mesh: outline.load_bevy_mesh(meshes).into(),
                                material: materials.add(ColorMaterial::from_color(
                                    Color::linear_rgba(0., 0., 0., 0.),
                                )),
                                transform: Transform::from_translation(Vec2::ZERO.extend(2.0)),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            Grid,
                            OverlayLayer2,
                        ))
                        .id();

                    let outline_entity = commands
                        .spawn((
                            Name::new(format!("Chunk Outline {chunk_ijk:?}")),
                            celestial_chunk_id,
                            // This will enable frustum culling via ViewVisibility by making a
                            // transparent mesh
                            // which will enforce bounds
                            MaterialMesh2dBundle {
                                mesh: outline.load_bevy_mesh(meshes).into(),
                                material: materials.add(ColorMaterial::from_color(
                                    Color::linear_rgba(0., 0., 0., 0.),
                                )),
                                transform: Transform::from_translation(Vec2::ZERO.extend(3.0)),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            GizmoDrawableLoop::new(outline, RED.into()),
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
                        visibility: Visibility::Visible,
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
        let wireframe_group = commands
            .spawn((
                Name::new("Cell Grids"),
                SpatialBundle {
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                GridGroup,
            ))
            .id();
        commands.entity(wireframe_group).push_children(&wireframes);
        commands
            .entity(celestial_id)
            .push_children(&[wireframe_group]);

        // Create an outlines entity parented to the celestial
        // which itself is the parent to all the other outline entities
        // this enables you to change their visibility easier with egui inspector
        // and cleans up the hierarchy
        let outline_group = commands
            .spawn((
                Name::new("Chunk Outlines"),
                SpatialBundle {
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                OutlineGroup,
            ))
            .id();
        commands.entity(outline_group).push_children(&outlines);
        commands
            .entity(celestial_id)
            .push_children(&[outline_group]);

        // And create events
        commands
            .entity(celestial_id)
            .insert(On::<Pointer<Down>>::send_event::<SelectCelestial>());

        // Parent the chunks to the celestial
        let chunk_group = commands
            .spawn((
                Name::new("Chunks"),
                SpatialBundle {
                    visibility: Visibility::Inherited,
                    ..Default::default()
                },
                ChunkGroup,
            ))
            .id();
        commands.entity(chunk_group).push_children(&chunks);
        commands.entity(celestial_id).push_children(&[chunk_group]);

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
    #[call_log_once]
    pub fn process_system(
        mut celestial_query: Query<(&mut Data, &mut Mass), Without<ChunkGroup>>,
        time: Res<Time>,
        frame: Res<FrameCount>,
    ) {
        // Process each celestial
        for (mut celestial, mut mass) in &mut celestial_query {
            trace_once!("Processing celestial");
            celestial.process(Clock::new(time.as_generic(), frame.as_ref().to_owned()));

            // Update the mass
            mass.0 = celestial.element_dir().total_mass().0;
        }
    }

    #[call_log_once]
    pub fn queue_materials_system(
        mut commands: Commands,
        mut celestial_query: Query<&mut Data, (Without<ChunkGroup>, Without<Chunk>)>,
        chunk_groups: Query<&Parent, (With<ChunkGroup>, Without<Chunk>)>,
        chunks: Query<
            (Entity, &Parent, &ChunkIjkComponent, &ViewVisibility),
            (Without<ChunkGroup>, With<Chunk>),
        >,
        asset_server: Res<AssetServer>,
    ) {
        let mut asset_updates: HashMap<AssetId<Image>, (Entity, Handle<Image>)> = HashMap::new();
        for (chunk_id, chunk_group_id, chunk_ijk, visibility) in &chunks {
            if visibility.get() {
                let chunk_group_parent = chunk_groups
                    .get(chunk_group_id.get())
                    .expect("Chunk groups are always parents of chunks");
                let mut celestial_data = celestial_query
                    .get_mut(chunk_group_parent.get())
                    .expect("Celestials are always parents of chunk groups");
                if let Some(mut texture) = celestial_data
                    .element_dir_mut()
                    .get_new_texture(chunk_ijk.0)
                {
                    let bevy_image = texture
                        .texture
                        .take()
                        .expect("Expected the texture to contain an image");

                    let handle = asset_server.add(bevy_image.to_bevy_image());
                    asset_updates.insert(handle.id(), (chunk_id, handle));
                    trace_once!("Updating material image");
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
    #[call_log_once]
    pub fn draw_materials_system(
        falling_sand_materials: Query<&Handle<ColorMaterial>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut asset_events: EventReader<AssetEvent<Image>>,
        mut asset_updates: ResMut<UpdatedTextures>,
    ) {
        // Process asset events
        for event in asset_events.read() {
            if let AssetEvent::<Image>::LoadedWithDependencies { id } = event {
                // Expect that the event id is present in asset_updates
                if let Some((chunk_id, image_handle)) = asset_updates.0.remove(id) {
                    // Expect to retrieve the material handle for the chunk
                    let material_handle = falling_sand_materials
                        .get(chunk_id)
                        .expect("Expected to find material handle for the given chunk ID");

                    // Expect to get the material, and set the texture
                    let material = materials
                        .get_mut(material_handle)
                        .expect("Expected to find material and set the texture");

                    material.texture = Some(image_handle);
                    trace_once!("Updating material");
                }
            }
        }
    }

    /// Draw the wireframe of the celestials cells
    #[call_log_once]
    pub fn draw_wireframe_system(
        mut gizmos: Gizmos,
        query: Query<
            (
                &GizmoDrawableGrid,
                &GlobalTransform,
                &InheritedVisibility,
                &ViewVisibility,
            ),
            With<Grid>,
        >,
    ) {
        for (drawable, transform, inherited_visibility, view_visibility) in query.iter() {
            if inherited_visibility.get() && view_visibility.get() {
                drawable.draw_bevy_gizmo_grid(&mut gizmos, &transform.compute_transform());
            }
        }
    }

    /// Draw the outline of the celestials chunks
    #[call_log_once]
    pub fn draw_outline_system(
        mut gizmos: Gizmos,
        query: Query<
            (
                &GizmoDrawableLoop,
                &GlobalTransform,
                &InheritedVisibility,
                &ViewVisibility,
            ),
            With<Outline>,
        >,
    ) {
        for (drawable, transform, inherited_visibility, view_visibility) in query.iter() {
            if inherited_visibility.get() && view_visibility.get() {
                drawable.draw_bevy_gizmo_loop(&mut gizmos, &transform.compute_transform());
            }
        }
    }

    /// If the [`Camera`] is a child of a celestial [`Data`], make the [`Grid`] visible
    #[call_log_once]
    pub fn wireframe_visibility_system(
        mut wireframe_groups: Query<
            (&Parent, &mut Visibility),
            (Without<MainCamera>, With<GridGroup>, Without<Data>),
        >,
        celestials: Query<Entity, (Without<MainCamera>, Without<GridGroup>, With<Data>)>,
        camera: Query<&Parent, (With<MainCamera>, Without<GridGroup>, Without<Data>)>,
    ) {
        if let Ok(camera_parent) = camera.get_single() {
            for (wireframe_parent, mut wireframe_visibility) in &mut wireframe_groups {
                if let Ok(celestial) = celestials.get(wireframe_parent.get()) {
                    if camera_parent.get() == celestial {
                        if *wireframe_visibility != Visibility::Visible {
                            debug!("Wireframe changed to visible");
                            *wireframe_visibility = Visibility::Visible;
                        }
                    } else if *wireframe_visibility != Visibility::Hidden {
                        debug!("Wireframe changed to hidden");
                        *wireframe_visibility = Visibility::Hidden;
                    }
                }
            }
        }
    }

    /// If the [`Camera`] is a child of a celestial [`Data`], make the [`Outline`] visible
    #[call_log_once]
    pub fn outline_visibility_system(
        mut outline_groups: Query<
            (&Parent, &mut Visibility),
            (Without<MainCamera>, With<OutlineGroup>, Without<Data>),
        >,
        celestials: Query<Entity, (Without<MainCamera>, Without<OutlineGroup>, With<Data>)>,
        camera: Query<&Parent, (With<MainCamera>, Without<OutlineGroup>, Without<Data>)>,
    ) {
        if let Ok(camera_parent) = camera.get_single() {
            for (outline_parent, mut outline_visibility) in &mut outline_groups {
                if let Ok(celestial) = celestials.get(outline_parent.get()) {
                    if camera_parent.get() == celestial {
                        if *outline_visibility != Visibility::Visible {
                            debug!("Outline changed to visible");
                            *outline_visibility = Visibility::Visible;
                        }
                    } else if *outline_visibility != Visibility::Hidden {
                        debug!("Outline changed to hidden");
                        *outline_visibility = Visibility::Hidden;
                    }
                }
            }
        }
    }
}
