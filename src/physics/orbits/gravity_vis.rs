use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Asset, Assets},
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        entity::Entity,
        query::{With},
        system::{Commands, Query, ResMut},
    },
    hierarchy::BuildChildren,
    math::Vec2,
    reflect::TypePath,
    render::{
        mesh::{shape, Mesh},
        render_resource::{AsBindGroup, ShaderRef},
    },
    sprite::{Material2d, MaterialMesh2dBundle},
    transform::components::{Transform},
    window::Window,
};

use super::{
    components::{GravitationalField, Mass},
    nbody::G,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct GravityField;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct GravityFieldBindGroup {
    #[uniform(0)]
    g: f32,

    #[storage(1)]
    positions: Vec<Vec2>,

    #[storage(2)]
    masses: Vec<f32>,
}

impl Default for GravityFieldBindGroup {
    fn default() -> Self {
        GravityFieldBindGroup {
            g: G,
            positions: Vec::new(),
            masses: Vec::new(),
        }
    }
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

    fn specialize(
        _descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        Ok(())
    }
}

pub struct GravityFieldPlugin;

impl Plugin for GravityFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, GravityFieldPlugin::setup);
        app.add_systems(Update, GravityFieldPlugin::update_gravity_well);
    }
}

impl GravityFieldPlugin {
    // Spawn an entity using `CustomMaterial`.
    fn setup(
        mut commands: Commands,
        mut materials: ResMut<Assets<GravityFieldBindGroup>>,
        mut meshes: ResMut<Assets<Mesh>>,
        camera: Query<Entity, With<Camera2d>>,
        windows: Query<&Window>,
    ) {
        let window = windows.single();
        let window_size = Vec2::new(window.width(), window.height());
        let entity = commands
            .spawn((
                MaterialMesh2dBundle {
                    material: materials.add(GravityFieldBindGroup::default()),
                    mesh: meshes.add(shape::Quad::new(window_size).into()).into(),
                    ..Default::default()
                },
                GravityField,
            ))
            .id();
        // Add the entity as a child of the camera.
        for camera in camera.iter() {
            commands.entity(camera).push_children(&[entity]);
        }
    }

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
}
