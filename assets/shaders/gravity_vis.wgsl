// This is a compute shader implementing leapfrog integration over a set of bodies
// Some bodies emit gravity, others do not
// Bodies which emit gravity effect each other
// Bodies which do not emit gravity are only effected by bodies which do emit gravity

#import orbiting_sand_rust::math::fast_length_2d;

@group(1) @binding(0) var<storage, read> positions: array<vec2<f32>>;
@group(2) @binding(1) var<storage, read> masses: array<f32>;

fn computeGravitationalVector(pos1: vec2<f32>, pos2: vec2<f32>, mass2: f32) -> vec2<f32> {
    let r = pos2.xy - pos1.xy;
    return (r * mass2) / (fast_length_2d(r) * dot(r, r));
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
        let force = computeGravitationalVector(vec3<f32>(pos, 0.0), vec3<f32>(grav_bodies[i].position, 0.0), grav_bodies[i].mass);
        totalForce += force;
    }

    // Calculate the magnitude of the total gravitational force
    let forceMagnitude = length(totalForce);

    // Discretize this magnitude to create contour lines
    let levels = 20.0; // Number of contour levels
    let level = floor(mod(forceMagnitude * 1000.0, levels));
    let colorIntensity = level / levels;

    // Color mapping based on contour level
    // Alpha should be the intensity
    let color = vec4<f32>(1.0, 0.0, 0.0, colorIntensity);

    return color;
}