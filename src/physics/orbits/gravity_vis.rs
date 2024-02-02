//! This module creates a visualizer for gravity fields.
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Asset, Assets, Handle},
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, ResMut},
    },
    math::Vec2,
    reflect::TypePath,
    render::{
        mesh::{shape, Mesh},
        render_resource::{AsBindGroup, ShaderRef},
    },
    sprite::{Material2d, MaterialMesh2dBundle},
    transform::components::Transform,
    window::{Window, WindowResized},
};

use super::components::{GravitationalField, Mass};

/// Identifies that this entity is an image to be overlayed on the screen.
#[derive(Component, Debug, Clone)]
pub struct ImageOverlay {
    /// The priority of the image. High numbers are drawn on top of low numbers.
    pub priority: i32,
}

/// Identifies that this entity is a gravity field.
#[derive(Component, Debug, Clone)]
pub struct GravityField;

/// Binds the positions and masses of all gravitational bodies to the shader.
#[derive(AsBindGroup, Default, Debug, Clone, Asset, TypePath)]
pub struct GravityFieldBindGroup {
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
        app.add_systems(Startup, GravityFieldPlugin::setup);
        app.add_systems(Update, GravityFieldPlugin::update_gravity_well);
        app.add_systems(Update, GravityFieldPlugin::window_resized);
    }
}

impl GravityFieldPlugin {
    /// Create the gravity field entity.
    fn setup(
        mut commands: Commands,
        mut materials: ResMut<Assets<GravityFieldBindGroup>>,
        mut meshes: ResMut<Assets<Mesh>>,
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
            ImageOverlay { priority: 0 },
        ));
    }

    /// Update the gravity field overlay with the positions and masses of all gravitational bodies.
    fn update_gravity_well(
        mut materials: ResMut<Assets<GravityFieldBindGroup>>,
        gravity_bodies: Query<(&Transform, &Mass), With<GravitationalField>>,
    ) {
        let gravity_field = materials.iter_mut().next().unwrap().1;
        let mut positions = Vec::new();
        let mut masses = Vec::new();
        for (transform, mass) in gravity_bodies.iter() {
            positions.push(transform.translation.truncate());
            masses.push(mass.0);
        }
        gravity_field.positions = positions;
        gravity_field.masses = masses;
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
