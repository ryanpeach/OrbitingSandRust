// This is a compute shader implementing leapfrog integration over a set of bodies
// Some bodies emit gravity, others do not
// Bodies which emit gravity effect each other
// Bodies which do not emit gravity are only effected by bodies which do emit gravity

struct GravityField {
    g: f32;
};

@group(1) @binding(0) var<uniform> material: GravityField;
@group(1) @binding(1) var<storage, read> positions: array<vec2<f32>>;
@group(2) @binding(2) var<storage, read> masses: array<f32>;

fn computeGravitationalForce(pos1: vec2<f32>, pos2: vec2<f32>, mass2: f32) -> vec2<f32> {
    let r = pos2.xy - pos1.xy;
    let distanceSquared = dot(r, r);
    let forceMagnitude = material.g * mass2 / distanceSquared;
    let forceDirection = normalize(r);
    return forceDirection * forceMagnitude;
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    var totalForce: vec2<f32> = vec2<f32>(0.0, 0.0);
    let pos = fragCoord.xy; // Assuming fragCoord.xy gives us the position in some space where we calculate forces

    for (var i = 0u; i < grav_bodies.length(); i++) {
        let force = computeGravitationalForce(vec3<f32>(pos, 0.0), vec3<f32>(grav_bodies[i].position, 0.0), grav_bodies[i].mass);
        totalForce += force;
    }

    // Calculate the magnitude of the total gravitational force
    let forceMagnitude = length(totalForce);

    // Discretize this magnitude to create contour lines
    let levels = 20.0; // Number of contour levels
    let level = floor(mod(forceMagnitude * 1000.0, levels));
    let colorIntensity = level / levels;

    // Color mapping based on contour level
    let color = vec4<f32>(colorIntensity, colorIntensity, 1.0 - colorIntensity, 1.0);

    return color;
}