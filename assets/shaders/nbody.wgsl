// This is a compute shader implementing leapfrog integration over a set of bodies
// Some bodies emit gravity, others do not
// Bodies which emit gravity effect each other
// Bodies which do not emit gravity are only effected by bodies which do emit gravity

struct Body {
    index: u32,
    /// This is the generation of the entity this body cooresponds to in the bevy world
    generation: u32,
    mass: f32,
    position: vec2<f32>,
    velocity: vec2<f32>,
};

struct Uniforms {
    dt: f32,
    g: f32,
    min_distance_squared: f32,
};

@group(0) @binding(0) var<storage, read> original_grav_bodies: array<Body>;
@group(0) @binding(1) var<storage, read_write> grav_bodies: array<Body>;
@group(0) @binding(2) var<storage, read_write> no_grav_bodies: array<Body>;
@group(0) @binding(3) var<uniform> myUniforms: Uniforms;

fn computeGravitationalForce(pos1: vec2<f32>, mass1: f32, pos2: vec2<f32>, mass2: f32) -> vec2<f32> {
    let r = pos2 - pos1;
    var distanceSquared = dot(r, r);
    distanceSquared = max(distanceSquared, myUniforms.min_distance_squared);
    let forceMagnitude = myUniforms.g * mass1 * mass2 / distanceSquared;
    let forceDirection = normalize(r);
    return forceDirection * forceMagnitude;
}

fn half_step(this_body: Body) -> Body {
    var net_force = vec2<f32>(0.0, 0.0);
    for (var i = 0u; i < arrayLength(&original_grav_bodies); i = i + 1u) {
        if (this_body.index == original_grav_bodies[i].index && this_body.generation == original_grav_bodies[i].generation) {
            continue;
        }
        let force = computeGravitationalForce(this_body.position, this_body.mass, original_grav_bodies[i].position, original_grav_bodies[i].mass);
        net_force += force;
    }
    let new_velocity = this_body.velocity + net_force / this_body.mass * (myUniforms.dt / 2.0);
    return Body(this_body.index, this_body.generation, this_body.mass, this_body.position, new_velocity);
}

fn full_position_update(this_body: Body) -> Body {
    let new_position = this_body.position + this_body.velocity * myUniforms.dt;
    return Body(this_body.index, this_body.generation, this_body.mass, new_position, this_body.velocity);
}

@compute @workgroup_size(64)
fn grav_bodies_single_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let first_half_step_body = half_step(grav_bodies[id.x]);
    let full_step_body = full_position_update(first_half_step_body);
    let second_half_step_body = half_step(full_step_body);
    grav_bodies[id.x] = second_half_step_body;
}

@compute @workgroup_size(64)
fn no_grav_bodies_single_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let half_step_body = half_step(no_grav_bodies[id.x]);
    let full_step_body = full_position_update(half_step_body);
    let second_half_step_body = half_step(full_step_body);
    no_grav_bodies[id.x] = second_half_step_body;
}