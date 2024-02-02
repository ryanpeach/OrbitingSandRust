//! This module creates a visualizer for gravity fields.
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{load_internal_asset, Asset, AssetApp, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, ResMut, Resource},
    },
    math::Vec2,
    pbr::{Material, MaterialPlugin},
    reflect::TypePath,
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::{shape, Mesh},
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
        Render, RenderApp,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    transform::components::Transform,
    window::{Window, WindowResized},
};
use rand::distributions::uniform;

use super::{
    components::{GravitationalField, Mass, OrbitalPosition},
    nbody::MIN_DISTANCE_SQUARED,
};

/// Identifies that this entity is a gravity field.
#[derive(Component, Debug, Clone)]
pub struct GravityField;

/// The parameters for the gravity field shader.
#[derive(Debug, Clone, ShaderType)]
pub struct Parameters {
    min_distance: f32,
    max_mass: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            min_distance: MIN_DISTANCE_SQUARED.sqrt(),
            max_mass: 1.0,
        }
    }
}

/// Binds the positions and masses of all gravitational bodies to the shader.
#[derive(Asset, TypePath, AsBindGroup, Debug, Default, Clone)]
pub struct GravityFieldBindGroup {
    /// The parameters for the gravity field shader.
    #[uniform(0)]
    pub parameters: Parameters,

    /// The positions of each gravitational body.
    #[storage(1)]
    pub positions: Vec<Vec2>,

    /// The mass of each gravitational body.
    #[storage(2)]
    pub masses: Vec<f32>,
}

// All functions on `Material2d` have default impls. You only need to implement the
// functions that are relevant for your material.
impl Material2d for GravityFieldBindGroup {
    fn fragment_shader() -> ShaderRef {
        "shaders/gravity_vis.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }
}

/// Creates a gravity field overlay.
pub struct GravityFieldPlugin;

impl Plugin for GravityFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<GravitationalField>::default());
        app.add_plugins(ExtractComponentPlugin::<Mass>::default());
        app.add_plugins(ExtractComponentPlugin::<OrbitalPosition>::default());
        app.add_plugins(Material2dPlugin::<GravityFieldBindGroup>::default());
        app.add_systems(Startup, GravityFieldPlugin::setup);
        app.add_systems(Update, OrbitalPosition::follow_transform_system);
        app.add_systems(Update, GravityFieldPlugin::window_resized);
        app.add_systems(Update, GravityFieldPlugin::update_gravity_well);
    }
}

impl GravityFieldPlugin {
    /// Create the gravity field entity.
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<GravityFieldBindGroup>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();
        let window_size = Vec2::new(window.width(), window.height());
        commands.spawn((
            MaterialMesh2dBundle {
                material: materials.add(GravityFieldBindGroup::default()),
                mesh: meshes.add(shape::Quad::new(window_size).into()).into(),
                ..Default::default()
            },
            GravityField,
        ));
    }

    /// Update the gravity field overlay with the positions and masses of all gravitational bodies.
    fn update_gravity_well(
        mut materials: ResMut<Assets<GravityFieldBindGroup>>,
        gravity_bodies: Query<(&OrbitalPosition, &Mass), With<GravitationalField>>,
    ) {
        let gravity_field = materials.iter_mut().next().unwrap().1;
        let mut positions = Vec::new();
        let mut masses = Vec::new();
        let mut max_mass = 1.0;
        for (transform, mass) in gravity_bodies.iter() {
            positions.push(transform.position());
            masses.push(mass.0);
            if mass.0 > max_mass {
                max_mass = mass.0;
            }
        }
        gravity_field.positions = positions;
        gravity_field.masses = masses;
        gravity_field.parameters.max_mass = max_mass;
    }

    /// Update the size of the gravity field overlay when the window is resized.
    fn window_resized(
        mut meshes: ResMut<Assets<Mesh>>,
        windows: Query<&Window>,
        mut gravity_field: Query<&Handle<Mesh>, With<GravitationalField>>,
        _window_resized_event: EventReader<WindowResized>,
    ) {
        let window = windows.single();
        let window_size = Vec2::new(window.width(), window.height());
        for mesh in gravity_field.iter_mut() {
            let mesh = meshes.get_mut(mesh).unwrap();
            *mesh = shape::Quad::new(window_size).into();
        }
    }
}
