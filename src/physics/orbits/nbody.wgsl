// This is a compute shader implementing leapfrog integration over a set of bodies
// Some bodies emit gravity, others do not
// Bodies which emit gravity effect each other
// Bodies which do not emit gravity are only effected by bodies which do emit gravity

struct Body {
    entity: u64,
    mass: f64,
    velocity: vec2<f32>,
    position: vec2<f32>,
    thrust: vec2<f32>,
};

struct Uniforms {
    G: f32;
    MIN_DISTANCE_SQUARED: f32;
    dt: f32;
};

@group(0) @binding(0) var<storage, read_write> grav_bodies: array<Body>;
@group(0) @binding(1) var<storage, read_write> no_grav_bodies: array<Body>;
@group(1) @binding(0) var<uniform> myUniforms: Uniforms;

fn computeGravitationalForce(pos1: vec3<f32>, mass1: f32, pos2: vec3<f32>, mass2: f32) -> vec2<f32> {
    let r = pos2.xy - pos1.xy;
    var distanceSquared = lengthSquared(r);
    distanceSquared = max(distanceSquared, myUniforms.MIN_DISTANCE_SQUARED);

    // The gravitational constant G and masses are factored into the force magnitude
    let forceMagnitude = myUniforms.G * mass1 * mass2;

    // Calculate the force direction
    // Divide the displacement vector by distanceSquared squared (distance to the fourth power)
    let forceDirection = r / (distanceSquared * distanceSquared);

    // The final force vector is scaled by the inverse square of the distance
    return forceDirection * forceMagnitude / distanceSquared;
}

fn half_step(this_body: Body, other_bodies: array<Body>) -> Body {
    let mut net_force = vec3<f32>(0.0, 0.0, 0.0);
    for other_body in other_bodies {
        if (this_body.idx == other_body.idx) {
            continue;
        }
        let force = computeGravitationalForce(this_body.position, this_body.mass, other_body.position, other_body.mass);
        net_force += force;
    }
    this_body.velocity += (net_force + this_body.thrust) / this_body.mass * (myUniforms.dt / 2.0);
    return this_body;
}

fn full_position_update(this_body: Body) -> Body {
    this_body.position += this_body.velocity * dt;
    return this_body;
}

fn grav_bodies_single_step(id: u32) {
    let first_half_step_body = half_step(grav_bodies[id], grav_bodies);
    let full_step_body = full_position_update(first_half_step_body);
    let second_half_step_body = half_step(full_step_body, grav_bodies);
    grav_bodies[id] = second_half_step_body;
}

fn no_grav_bodies_single_step(id: u32) {
    let half_step_body = half_step(no_grav_bodies[id], grav_bodies);
    let full_step_body = full_position_update(half_step_body);
    let second_half_step_body = half_step(full_step_body, grav_bodies);
    no_grav_bodies[id] = second_half_step_body;
}

/// WARNING: It's important that workgroup_size matches the WORKGROUP_SIZE in the nbody.rs file
@compute @workgroup_size(64)
fn single_step(@builtin(global_invocation_id) id: vec3<u32>) {
    grav_bodies_single_step(id.x);
    no_grav_bodies_single_step(id.x);
}