//! NBody physics simulation
//! First attempt at using compute shaders in bevy

use std::{borrow::Cow, ops::Deref};

use super::components::{GravitationalField, Mass, OrbitalPosition, Velocity};
use bevy::{
    ecs::query::QueryIter,
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_graph::{self, RenderGraph},
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
            BufferVec, CachedComputePipelineId, CachedPipelineState, CommandEncoderDescriptor,
            ComputePassDescriptor, ComputePipelineDescriptor, MapMode, PipelineCache, ShaderStages,
            ShaderType, UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};
use bytemuck::{Pod, Zeroable};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wgpu::Maintain;

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
const MIN_DISTANCE_SQUARED: f32 = 100.0;
const G: f32 = 1.0e3;

/// WARNING: It's important that this matches the workgroup size in the compute shader
const WORKGROUP_SIZE: u32 = 64;
const MAX_NB_NO_GRAV_BODIES: u32 = 10000;
const MAX_NB_GRAV_BODIES: u32 = 10;

/// A body in the wgsl code
#[repr(C)]
#[derive(Debug, Copy, Clone, ShaderType, Pod, Zeroable)]
struct Body {
    /// This is the entity index this body cooresponds to in the bevy world
    index: u32,
    /// This is the generation of the entity this body cooresponds to in the bevy world
    generation: u32,
    mass: f32,
    position: Vec2,
    velocity: Vec2,
}

impl Body {
    fn entity(&self) -> Entity {
        let bits = (self.generation as u64) << 32 | self.index as u64;
        Entity::from_bits(bits)
    }

    fn convert_bytes(bytes: Vec<u8>) -> Vec<Self> {
        let slice = bytemuck::cast_slice(&bytes);
        slice.to_vec()
    }
}

/// Uniforms struct which contains most of the parameters for the simulation
#[repr(C)]
#[derive(Debug, Copy, Clone, ShaderType)]
struct Uniforms {
    pub dt: f32,
    pub g: f32,
    pub min_distance_squared: f32,
}

/// Plugin to set up nbody physics
pub struct NBodyPlugin;

impl Plugin for NBodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, OrbitalPosition::sync_with_transform_system);
        app.add_plugins(ExtractComponentPlugin::<Mass>::default());
        app.add_plugins(ExtractComponentPlugin::<Velocity>::default());
        app.add_plugins(ExtractComponentPlugin::<OrbitalPosition>::default());
        app.add_plugins(ExtractComponentPlugin::<GravitationalField>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NBodyBuffers>();
        render_app.init_resource::<NBodyCopyBuffers>();
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
        render_app.add_systems(Render, NBodyCopyBuffers::copy_system);
        // render_app.add_systems(Update, NBodyPlugin::update_buffers_from_query);
    }
}

/// Systems that go from buffers to world and back
/// Made both exclusive systems and query systems to play around with different implementations
impl NBodyPlugin {
    // fn update_buffers_from_world(world: &mut World) {
    //     let mut nbody_buffers = world.resource_mut::<NBodyBuffers>();
    //     let render_device = world.resource::<RenderDevice>();
    //     let render_queue = world.resource::<RenderQueue>();
    //     let mut no_grav_bodies_query = world
    //         .query_filtered::<(Entity, &Mass, &Velocity, &OrbitalPosition), Without<GravitationalField>>(
    //         );
    //     let no_grav_bodies_vec = Self::bodies_from_no_grav_entities(no_grav_bodies_query.iter(&world));
    //     let mut grav_bodies_query = world
    //         .query_filtered::<(Entity, &Mass, &Velocity, &OrbitalPosition), With<GravitationalField>>();
    //     let grav_bodies_vec =  Self::bodies_from_grav_entities(grav_bodies_query.iter(&world));

    //     let _ = &nbody_buffers.load(
    //         &render_device,
    //         &render_queue,
    //         no_grav_bodies_vec,
    //         grav_bodies_vec,
    //         0.0,
    //     );
    // }

    // fn update_buffers_from_query(
    //     no_grav_bodies_query: Query<
    //         (Entity, &Mass, &Velocity, &OrbitalPosition),
    //         Without<GravitationalField>,
    //     >,
    //     grav_bodies_query: Query<(Entity, &Mass, &Velocity, &OrbitalPosition), With<GravitationalField>>,
    //     render_device: Res<RenderDevice>,
    //     render_queue: Res<RenderQueue>,
    //     mut nbody_buffers: ResMut<NBodyBuffers>,
    // ) {
    //     let no_grav_bodies_vec = Self::bodies_from_no_grav_entities(no_grav_bodies_query.iter());
    //     let grav_bodies_vec = Self::bodies_from_grav_entities(grav_bodies_query.iter());

    //     let _ = &nbody_buffers.load(
    //         &render_device,
    //         &render_queue,
    //         no_grav_bodies_vec,
    //         grav_bodies_vec,
    //         0.0,
    //     );
    // }

    fn bodies_from_no_grav_entities(
        entities: QueryIter<
            '_,
            '_,
            (Entity, &Mass, &Velocity, &OrbitalPosition),
            Without<GravitationalField>,
        >,
    ) -> Vec<Body> {
        entities
            .map(|(entity, mass, velocity, position)| Body {
                index: entity.index(),
                generation: entity.generation(),
                mass: mass.0,
                position: position.0,
                velocity: velocity.0,
            })
            .collect::<Vec<_>>()
    }

    fn bodies_from_grav_entities(
        entities: QueryIter<
            '_,
            '_,
            (Entity, &Mass, &Velocity, &OrbitalPosition),
            With<GravitationalField>,
        >,
    ) -> Vec<Body> {
        entities
            .map(|(entity, mass, velocity, position)| Body {
                index: entity.index(),
                generation: entity.generation(),
                mass: mass.0,
                position: position.0,
                velocity: velocity.0,
            })
            .collect::<Vec<_>>()
    }

    // fn update_world_from_buffers(world: &mut World) {
    //     let values = world
    //         .resource::<NBodyReadBuffers>()
    //         .no_grav_bodies_buffer
    //         .values()
    //         .clone();
    //     for body in values.iter() {
    //         let idx = body.entity();
    //         {
    //             let mut transform = world.get_mut::<OrbitalPosition>(idx).unwrap();
    //             transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
    //             trace!("no_grav_bodies.body.position: {:?}", body.position)
    //         }
    //         {
    //             let mut velocity = world.get_mut::<Velocity>(idx).unwrap();
    //             velocity.0 = body.velocity;
    //         }
    //     }
    //     let values = world
    //         .resource::<NBodyReadBuffers>()
    //         .grav_bodies_buffer
    //         .values()
    //         .clone();
    //     for body in values.iter() {
    //         let idx = body.entity();
    //         {
    //             let mut transform = world.get_mut::<OrbitalPosition>(idx).unwrap();
    //             transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
    //             trace!("grav_bodies.body.position: {:?}", body.position)
    //         }
    //         {
    //             let mut velocity = world.get_mut::<Velocity>(idx).unwrap();
    //             velocity.0 = body.velocity;
    //         }
    //     }
    // }

    // fn update_query_from_buffers(
    //     nbody_buffers: ResMut<NBodyReadBuffers>,
    //     mut no_grav_bodies_query: Query<
    //         (Entity, &Mass, &Velocity, &OrbitalPosition),
    //         Without<GravitationalField>,
    //     >,
    //     mut grav_bodies_query: Query<
    //         (Entity, &Mass, &Velocity, &OrbitalPosition),
    //         With<GravitationalField>,
    //     >,
    // ) {
    //     let values = nbody_buffers.no_grav_bodies_buffer.values().clone();
    //     for body in values.iter() {
    //         let idx = body.entity();
    //         {
    //             let mut transform = no_grav_bodies_query
    //                 .get_component_mut::<OrbitalPosition>(idx)
    //                 .unwrap();
    //             transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
    //             trace!("no_grav_bodies.body.position: {:?}", body.position)
    //         }
    //         {
    //             let mut velocity = no_grav_bodies_query
    //                 .get_component_mut::<Velocity>(idx)
    //                 .unwrap();
    //             velocity.0 = body.velocity;
    //         }
    //     }
    //     let values = nbody_buffers.grav_bodies_buffer.values().clone();
    //     for body in values.iter() {
    //         let idx = body.entity();
    //         {
    //             let mut transform = grav_bodies_query
    //                 .get_component_mut::<OrbitalPosition>(idx)
    //                 .unwrap();
    //             transform.translation = Vec3::new(body.position.x, body.position.y, 0.0);
    //             trace!("grav_bodies.body.position: {:?}", body.position)
    //         }
    //         {
    //             let mut velocity = grav_bodies_query
    //                 .get_component_mut::<Velocity>(idx)
    //                 .unwrap();
    //             velocity.0 = body.velocity;
    //         }
    //     }
    // }
}

// ==============================
// Shader Code
// ==============================

/// These are the buffers which actually store the data
#[derive(Resource)]
struct NBodyBuffers {
    pub original_grav_bodies_buffer: BufferVec<Body>,
    pub grav_bodies_buffer: BufferVec<Body>,
    pub no_grav_bodies_buffer: BufferVec<Body>,
    pub uniform_buffer: UniformBuffer<Uniforms>,
}

impl NBodyBuffers {
    fn clear(&mut self) {
        self.original_grav_bodies_buffer.clear();
        self.grav_bodies_buffer.clear();
        self.no_grav_bodies_buffer.clear();
    }

    fn write(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.original_grav_bodies_buffer
            .write_buffer(render_device, render_queue);
        self.grav_bodies_buffer
            .write_buffer(render_device, render_queue);
        self.no_grav_bodies_buffer
            .write_buffer(render_device, render_queue);
        self.uniform_buffer
            .write_buffer(render_device, render_queue);
    }

    fn reserve(&mut self, render_device: &RenderDevice) {
        self.original_grav_bodies_buffer
            .reserve(MAX_NB_GRAV_BODIES as usize, render_device);
        self.grav_bodies_buffer
            .reserve(MAX_NB_GRAV_BODIES as usize, render_device);
        self.no_grav_bodies_buffer
            .reserve(MAX_NB_NO_GRAV_BODIES as usize, render_device);
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
        self.reserve(render_device);
        if grav_bodies.len() > MAX_NB_GRAV_BODIES as usize {
            panic!("Too many grav_bodies");
        } else {
            trace!("grav_bodies.len(): {}", grav_bodies.len());
        }
        if no_grav_bodies.len() > MAX_NB_NO_GRAV_BODIES as usize {
            panic!("Too many no_grav_bodies");
        } else {
            trace!("no_grav_bodies.len(): {}", no_grav_bodies.len());
        }
        self.original_grav_bodies_buffer.extend(grav_bodies.clone());
        self.grav_bodies_buffer.extend(grav_bodies);
        self.no_grav_bodies_buffer.extend(no_grav_bodies);
        self.uniform_buffer.set(Uniforms {
            dt,
            g: G,
            min_distance_squared: MIN_DISTANCE_SQUARED,
        });
        self.write(render_device, render_queue);
    }
}

/// Create the buffers
impl FromWorld for NBodyBuffers {
    fn from_world(_world: &mut World) -> Self {
        let original_grav_bodies_buffer =
            BufferVec::<Body>::new(BufferUsages::STORAGE | BufferUsages::MAP_WRITE);

        // Create a buffer for grav_bodies
        let grav_bodies_buffer = BufferVec::<Body>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
        );

        // Create a buffer for no_grav_bodies
        let no_grav_bodies_buffer = BufferVec::<Body>::new(
            BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
        );

        // Create a buffer for uniforms
        let uniform_buffer = Uniforms {
            dt: 0.0,
            g: G,
            min_distance_squared: MIN_DISTANCE_SQUARED,
        };

        NBodyBuffers {
            original_grav_bodies_buffer,
            grav_bodies_buffer,
            no_grav_bodies_buffer,
            uniform_buffer: uniform_buffer.into(),
        }
    }
}

/// These are what you use to read data out of the buffers
#[derive(Resource)]
struct NBodyCopyBuffers {
    pub grav_bodies_buffer: BufferVec<Body>,
    pub no_grav_bodies_buffer: BufferVec<Body>,
}

/// Create the buffers
impl FromWorld for NBodyCopyBuffers {
    fn from_world(_world: &mut World) -> Self {
        // Create a buffer for grav_bodies
        let grav_bodies_buffer =
            BufferVec::<Body>::new(BufferUsages::COPY_DST | BufferUsages::MAP_READ);

        // Create a buffer for no_grav_bodies
        let no_grav_bodies_buffer =
            BufferVec::<Body>::new(BufferUsages::COPY_DST | BufferUsages::MAP_READ);

        NBodyCopyBuffers {
            grav_bodies_buffer,
            no_grav_bodies_buffer,
        }
    }
}

impl NBodyCopyBuffers {
    fn clear(&mut self) {
        self.grav_bodies_buffer.clear();
        self.no_grav_bodies_buffer.clear();
    }

    fn reserve(&mut self, render_device: &RenderDevice) {
        self.grav_bodies_buffer
            .reserve(MAX_NB_GRAV_BODIES as usize, render_device);
        self.no_grav_bodies_buffer
            .reserve(MAX_NB_NO_GRAV_BODIES as usize, render_device);
    }

    fn copy_system(
        nbody_copy_buffers: Res<NBodyCopyBuffers>,
        nbody_buffers: Res<NBodyBuffers>,
        mut grav_bodies: Query<
            (Entity, &mut Velocity, &mut OrbitalPosition),
            With<GravitationalField>,
        >,
        mut no_grav_bodies: Query<
            (Entity, &mut Velocity, &mut OrbitalPosition),
            Without<GravitationalField>,
        >,
        queue: Res<RenderQueue>,
        device: Res<RenderDevice>,
    ) {
        if nbody_copy_buffers.grav_bodies_buffer.buffer().is_none() {
            error!("Buffers not loaded yet");
            return;
        }
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let grav_bodies_buffer = nbody_buffers.grav_bodies_buffer.buffer().unwrap();
        let grav_bodies_copy_buffer = nbody_copy_buffers.grav_bodies_buffer.buffer().unwrap();
        encoder.copy_buffer_to_buffer(
            grav_bodies_buffer,
            0,
            grav_bodies_copy_buffer,
            0,
            grav_bodies_buffer.size(),
        );
        let no_grav_bodies_buffer = nbody_buffers.no_grav_bodies_buffer.buffer().unwrap();
        let no_grav_bodies_copy_buffer = nbody_copy_buffers.no_grav_bodies_buffer.buffer().unwrap();
        encoder.copy_buffer_to_buffer(
            no_grav_bodies_buffer,
            0,
            no_grav_bodies_copy_buffer,
            0,
            grav_bodies_buffer.size(),
        );
        queue.submit([encoder.finish()]);

        let grav_bodies_copy_buffer_slice = grav_bodies_copy_buffer.slice(..);
        grav_bodies_copy_buffer_slice.map_async(MapMode::Read, move |result| {
            let err = result.err();
            if err.is_some() {
                panic!("{}", err.unwrap().to_string());
            }
        });
        let no_grav_bodies_copy_buffer_slice = no_grav_bodies_copy_buffer.slice(..);
        no_grav_bodies_copy_buffer_slice.map_async(MapMode::Read, move |result| {
            let err = result.err();
            if err.is_some() {
                panic!("{}", err.unwrap().to_string());
            }
        });

        device.poll(Maintain::Wait);

        let grav_bodies_copy_buffer_data = Body::convert_bytes(Vec::from(
            grav_bodies_copy_buffer_slice.get_mapped_range().deref(),
        ));
        let no_grav_bodies_copy_buffer_data = Body::convert_bytes(Vec::from(
            no_grav_bodies_copy_buffer_slice.get_mapped_range().deref(),
        ));

        let grav_bodies_copy_buffer_data_hashmap: HashMap<Entity, Body> =
            grav_bodies_copy_buffer_data
                .par_iter()
                .map(|body| (body.entity(), *body))
                .collect();
        let no_grav_bodies_copy_buffer_data_hashmap: HashMap<Entity, Body> =
            no_grav_bodies_copy_buffer_data
                .par_iter()
                .map(|body| (body.entity(), *body))
                .collect();

        grav_bodies
            .par_iter_mut()
            .for_each(|(entity, mut velocity, mut position)| {
                let body = grav_bodies_copy_buffer_data_hashmap.get(&entity).unwrap();
                position.0 = body.position;
                velocity.0 = body.velocity;
            });
        no_grav_bodies
            .par_iter_mut()
            .for_each(|(entity, mut velocity, mut position)| {
                if let Some(body) = no_grav_bodies_copy_buffer_data_hashmap.get(&entity) {
                    position.0 = body.position;
                    velocity.0 = body.velocity;
                } else {
                    error!("No body found for entity: {:?}", entity);
                }
            });
    }
}

/// TODO: Seperate layouts for seperate compute shaders for efficiency
#[derive(Clone)]
struct NBodyBindGroupLayouts(pub BindGroupLayout);

impl NBodyBindGroupLayouts {
    fn new(render_device: &RenderDevice) -> Self {
        let group_0 = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        NBodyBindGroupLayouts(group_0)
    }
}

#[derive(Resource)]
struct NBodyBindGroups(pub BindGroup);

impl NBodyBindGroups {
    #[allow(clippy::too_many_arguments)]
    fn prepare_bind_groups(
        mut commands: Commands,
        render_device: Res<RenderDevice>,
        render_queue: Res<RenderQueue>,
        mut nbody_buffers: ResMut<NBodyBuffers>,
        mut read_buffers: ResMut<NBodyCopyBuffers>,
        no_grav_entities_query: Query<
            (Entity, &Mass, &Velocity, &OrbitalPosition),
            Without<GravitationalField>,
        >,
        grav_entities_query: Query<
            (Entity, &Mass, &Velocity, &OrbitalPosition),
            With<GravitationalField>,
        >,
        time: Res<Time>,
    ) {
        let nbody_layouts = NBodyBindGroupLayouts::new(&render_device);
        nbody_buffers.load(
            &render_device,
            &render_queue,
            NBodyPlugin::bodies_from_no_grav_entities(no_grav_entities_query.iter()),
            NBodyPlugin::bodies_from_grav_entities(grav_entities_query.iter()),
            time.delta_seconds(),
        );
        read_buffers.reserve(&render_device);

        // Create a BindingResource from BufferVec
        let original_grav_bodies_buffer_binding = nbody_buffers
            .original_grav_bodies_buffer
            .buffer()
            .unwrap()
            .as_entire_buffer_binding();
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
        let uniform_buffer_binding = nbody_buffers
            .uniform_buffer
            .buffer()
            .unwrap()
            .as_entire_buffer_binding();

        // Create a BindGroupEntry for the BufferVec
        let original_grav_bodies_buffer_entry = BindGroupEntry {
            binding: 0, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(original_grav_bodies_buffer_binding),
        };
        let grav_bodies_buffer_entry = BindGroupEntry {
            binding: 1, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(grav_bodies_buffer_binding),
        };
        let no_grav_bodies_buffer_entry = BindGroupEntry {
            binding: 2, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(no_grav_bodies_buffer_binding),
        };
        let uniform_buffer_entry = BindGroupEntry {
            binding: 3, // The binding number must match the shader's binding number
            resource: BindingResource::Buffer(uniform_buffer_binding),
        };

        // Create the BindGroup using the entry
        let group_0 = render_device.create_bind_group(
            None,
            &nbody_layouts.0,
            &[
                original_grav_bodies_buffer_entry,
                grav_bodies_buffer_entry,
                no_grav_bodies_buffer_entry,
                uniform_buffer_entry,
            ],
        );
        let out = NBodyBindGroups(group_0);
        commands.insert_resource(out);
    }
}

#[derive(Resource)]
pub struct NBodyPipeline {
    nbody_bind_group_layouts: NBodyBindGroupLayouts,
    grav_bodies_single_step: CachedComputePipelineId,
    no_grav_bodies_single_step: CachedComputePipelineId,
}

impl FromWorld for NBodyPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load the compute shader
        let asset_server = world.resource::<AssetServer>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = asset_server.load("shaders/nbody.wgsl");
        let render_device = world.resource::<RenderDevice>();
        let layouts = NBodyBindGroupLayouts::new(render_device);
        let grav_bodies_single_step =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("grav_bodies_single_step")),
                shader: shader.clone(),
                entry_point: Cow::from("grav_bodies_single_step"),
                layout: vec![layouts.0.clone()],
                push_constant_ranges: vec![],
                shader_defs: vec![],
            });
        let no_grav_bodies_single_step =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("no_grav_bodies_single_step")),
                shader: shader.clone(),
                entry_point: Cow::from("no_grav_bodies_single_step"),
                layout: vec![layouts.0.clone()],
                push_constant_ranges: vec![],
                shader_defs: vec![],
            });
        NBodyPipeline {
            nbody_bind_group_layouts: layouts,
            grav_bodies_single_step,
            no_grav_bodies_single_step,
        }
    }
}

#[derive(Default)]
enum NBodyState {
    #[default]
    Loading1,
    Run1,
    Loading2,
    Run2,
    Done,
}

#[derive(Default)]
struct NBodyNode {
    state: NBodyState,
}

impl render_graph::Node for NBodyNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<NBodyPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            NBodyState::Loading1 => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.grav_bodies_single_step)
                {
                    self.state = NBodyState::Run1;
                }
            }
            NBodyState::Run1 => {
                self.state = NBodyState::Loading2;
            }
            NBodyState::Loading2 => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.no_grav_bodies_single_step)
                {
                    self.state = NBodyState::Run2;
                }
            }
            NBodyState::Run2 => {
                self.state = NBodyState::Done;
            }
            NBodyState::Done => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<NBodyBindGroups>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<NBodyPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            NBodyState::Loading1 => {}
            NBodyState::Run1 => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.grav_bodies_single_step)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(MAX_NB_GRAV_BODIES / WORKGROUP_SIZE, 1, 1)
            }
            NBodyState::Loading2 => {}
            NBodyState::Run2 => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.no_grav_bodies_single_step)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(MAX_NB_NO_GRAV_BODIES / WORKGROUP_SIZE, 1, 1)
            }
            NBodyState::Done => {}
        }

        Ok(())
    }
}
