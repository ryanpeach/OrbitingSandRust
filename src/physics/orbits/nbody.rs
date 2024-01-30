use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph},
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages, BufferVec,
            CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor,
            ComputePipelineDescriptor, IntoBinding, PipelineCache,
            ShaderStages, ShaderType, UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};

use super::components::{GravitationalField, Mass, Velocity};

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
const MIN_DISTANCE_SQUARED: f32 = 1.0;
const G: f32 = 1.0;
const WORKGROUP_SIZE: u32 = 64;

/// A body in the wgsl code
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Body {
    entity: u64,
    mass: f64,
    position: Vec2,
    velocity: Vec2,
    thrust: Vec2,
}

impl Body {
    fn entity(&self) -> Entity {
        Entity::from_bits(self.entity)
    }
}

// Uniforms struct
#[repr(C)]
#[derive(Debug, Copy, Clone, ShaderType)]
struct Uniforms {
    g: f32,
    min_distance_squared: f32,
    dt: f32,
}

// Plugin to set up nbody physics
pub struct NBodyPlugin;

impl Plugin for NBodyPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NBodyBuffers>();
        render_app.add_systems(
            Render,
            NBodyBindGroups::prepare_bind_groups.in_set(RenderSet::PrepareBindGroups),
        );
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("nbody", NBodyNode::default());
        render_graph.add_node_edge("nbody", bevy::render::main_graph::node::CAMERA_DRIVER);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NBodyPipeline>();
    }
}

impl NBodyPlugin {
    fn update_system(
        no_grav_bodies: Query<
            (Entity, &Mass, &mut Velocity, &mut Transform),
            Without<GravitationalField>,
        >,
        grav_bodies: Query<
            (Entity, &Mass, &mut Velocity, &mut Transform),
            With<GravitationalField>,
        >,
        render_device: Res<RenderDevice>,
        render_queue: Res<RenderQueue>,
        mut nbody_buffers: ResMut<NBodyBuffers>,
    ) {
        let no_grav_bodies_vec = no_grav_bodies
            .iter()
            .map(|(entity, mass, velocity, transform)| Body {
                entity: entity.to_bits(),
                mass: mass.0 as f64,
                position: transform.translation.truncate(),
                velocity: velocity.0,
                thrust: Vec2::ZERO,
            })
            .collect::<Vec<_>>();
        let grav_bodies_vec = grav_bodies
            .iter()
            .map(|(entity, mass, velocity, transform)| Body {
                entity: entity.to_bits(),
                mass: mass.0 as f64,
                position: transform.translation.truncate(),
                velocity: velocity.0,
                thrust: Vec2::ZERO,
            })
            .collect::<Vec<_>>();

        let _ = &nbody_buffers.load(
            &render_device,
            &render_queue,
            no_grav_bodies_vec,
            grav_bodies_vec,
            0.0,
        );
    }
}

#[derive(Resource)]
struct NBodyBuffers {
    pub grav_bodies_buffer: BufferVec<Body>,
    pub no_grav_bodies_buffer: BufferVec<Body>,
    pub uniform_buffer: UniformBuffer<Uniforms>,
}

impl NBodyBuffers {
    fn clear(&mut self) {
        self.grav_bodies_buffer.clear();
        self.no_grav_bodies_buffer.clear();
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.grav_bodies_buffer
            .write_buffer(render_device, render_queue);
        self.no_grav_bodies_buffer
            .write_buffer(render_device, render_queue);
        self.uniform_buffer
            .write_buffer(render_device, render_queue);
    }

    pub fn load(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        no_grav_bodies: Vec<Body>,
        grav_bodies: Vec<Body>,
        dt: f32,
    ) {
        self.clear();
        self.grav_bodies_buffer.extend(grav_bodies);
        self.no_grav_bodies_buffer.extend(no_grav_bodies);
        self.uniform_buffer = Uniforms {
            g: G,
            min_distance_squared: MIN_DISTANCE_SQUARED,
            dt,
        }
        .into();
        self.write(render_device, render_queue);
    }
}

impl FromWorld for NBodyBuffers {
    fn from_world(_world: &mut World) -> Self {
        // Create a buffer for grav_bodies
        let grav_bodies_buffer = BufferVec::<Body>::new(BufferUsages::STORAGE);

        // Create a buffer for no_grav_bodies
        let no_grav_bodies_buffer = BufferVec::<Body>::new(BufferUsages::STORAGE);

        // Create a buffer for uniforms
        let uniform_buffer = Uniforms {
            g: G,
            min_distance_squared: MIN_DISTANCE_SQUARED,
            dt: 0.0,
        };

        NBodyBuffers {
            grav_bodies_buffer,
            no_grav_bodies_buffer,
            uniform_buffer: uniform_buffer.into(),
        }
    }
}

#[derive(Clone)]
struct NBodyBindGroupLayouts {
    pub group_0: BindGroupLayout,
    pub group_1: BindGroupLayout,
}

impl NBodyBindGroupLayouts {
    fn new(render_device: &RenderDevice) -> Self {
        let group_0 = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let group_1 = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });
        NBodyBindGroupLayouts { group_0, group_1 }
    }
}

#[derive(Resource)]
struct NBodyBindGroups {
    pub group_0: BindGroup,
    pub group_1: BindGroup,
}

impl NBodyBindGroups {
    fn prepare_bind_groups(
        mut commands: Commands,
        render_device: Res<RenderDevice>,
        nbody_buffers: Res<NBodyBuffers>,
    ) {
        let nbody_layouts = NBodyBindGroupLayouts::new(&render_device);

        // Create a BindingResource from BufferVec
        if nbody_buffers.grav_bodies_buffer.buffer().is_none() {
            return;
        }
        let grav_bodies_buffer_binding = nbody_buffers
            .grav_bodies_buffer
            .buffer()
            .unwrap()
            .as_entire_buffer_binding();
        let no_grav_bodies_buffer_binding = nbody_buffers
            .no_grav_bodies_buffer
            .buffer()
            .unwrap()
            .as_entire_buffer_binding();
        let uniform_buffer_binding = nbody_buffers.uniform_buffer.into_binding();

        // Create a BindGroupEntry for the BufferVec
        let grav_bodies_buffer_entry = BindGroupEntry {
            binding: 1, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(grav_bodies_buffer_binding),
        };
        let no_grav_bodies_buffer_entry = BindGroupEntry {
            binding: 2, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(no_grav_bodies_buffer_binding),
        };
        let uniform_buffer_entry = BindGroupEntry {
            binding: 0, // The binding number must match the shader's binding number
            resource: uniform_buffer_binding,
        };

        // Create the BindGroup using the entry
        let group_0 = render_device.create_bind_group(
            None,
            &nbody_layouts.group_0,
            &[grav_bodies_buffer_entry, no_grav_bodies_buffer_entry],
        );
        let group_1 =
            render_device.create_bind_group(None, &nbody_layouts.group_1, &[uniform_buffer_entry]);
        let out = NBodyBindGroups { group_0, group_1 };
        commands.insert_resource(out);
    }
}

#[derive(Resource)]
pub struct NBodyPipeline {
    nbody_bind_group_layouts: NBodyBindGroupLayouts,
    compute_shader: CachedComputePipelineId,
}

impl FromWorld for NBodyPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load the compute shader
        let asset_server = world.resource::<AssetServer>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = asset_server.load("./nbody.wgsl");
        let render_device = world.resource::<RenderDevice>();
        let layouts = NBodyBindGroupLayouts::new(render_device);
        let compute_shader = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("single_step")),
            shader: shader.clone(),
            entry_point: Cow::from("single_step"),
            layout: vec![layouts.group_0.clone(), layouts.group_1.clone()],
            push_constant_ranges: vec![],
            shader_defs: vec![],
        });
        NBodyPipeline {
            nbody_bind_group_layouts: layouts,
            compute_shader,
        }
    }
}

enum NBodyState {
    Loading,
    Running,
    PostRun,
}

struct NBodyNode {
    state: NBodyState,
}

impl Default for NBodyNode {
    fn default() -> Self {
        Self {
            state: NBodyState::Loading,
        }
    }
}

impl render_graph::Node for NBodyNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<NBodyPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let _render_device = world.resource::<RenderDevice>();
        let _render_queue = world.resource::<RenderQueue>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            NBodyState::Loading => {
                if let CachedPipelineState::Queued =
                    pipeline_cache.get_compute_pipeline_state(pipeline.compute_shader)
                {
                    self.state = NBodyState::Running;
                }
            }
            NBodyState::Running => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.compute_shader)
                {
                    self.state = NBodyState::PostRun;
                }
            }
            NBodyState::PostRun => {
                let values = world
                    .resource::<NBodyBuffers>()
                    .no_grav_bodies_buffer
                    .values()
                    .clone();
                for body in values.iter() {
                    let idx = Entity::from_bits(body.entity);
                    {
                        let mut transform = world.get_mut::<Transform>(idx).unwrap();
                        transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
                    }
                    {
                        let mut velocity = world.get_mut::<Velocity>(idx).unwrap();
                        velocity.0 = body.velocity;
                    }
                }
                let values = world
                    .resource::<NBodyBuffers>()
                    .grav_bodies_buffer
                    .values()
                    .clone();
                for body in values.iter() {
                    let idx = Entity::from_bits(body.entity);
                    {
                        let mut transform = world.get_mut::<Transform>(idx).unwrap();
                        transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
                    }
                    {
                        let mut velocity = world.get_mut::<Velocity>(idx).unwrap();
                        velocity.0 = body.velocity;
                    }
                }
            }
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !world.contains_resource::<NBodyBindGroups>() {
            return Ok(());
        }
        let texture_bind_group = world.resource::<NBodyBindGroups>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<NBodyPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &texture_bind_group.group_0, &[]);
        pass.set_bind_group(1, &texture_bind_group.group_1, &[]);

        // select the pipeline based on the current state
        match self.state {
            NBodyState::Loading => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.compute_shader)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(WORKGROUP_SIZE, 1, 1);
            }
            NBodyState::Running => {}
            NBodyState::PostRun => {}
        }

        Ok(())
    }
}
