// This is a compute shader implementing leapfrog integration over a set of bodies
// Some bodies emit gravity, others do not
// Bodies which emit gravity effect each other
// Bodies which do not emit gravity are only effected by bodies which do emit gravity

#import bevy_pbr::forward_io::VertexOutput;

struct Parameters {
    min_distance: f32,
    max_mass: f32,
}

@group(1) @binding(0) var<uniform> parameters: Parameters;
@group(1) @binding(1) var<storage, read> positions: array<vec2<f32>>;
@group(1) @binding(2) var<storage, read> masses: array<f32>;

fn computeGravitationalVector(pos1: vec2<f32>, pos2: vec2<f32>, mass2: f32) -> vec2<f32> {
    let r = pos2.xy - pos1.xy;
    return (r * mass2) / (max(fast_length_2d(r), parameters.min_distance) * dot(r, r));
}

// [Drobot2014a] Low Level Optimizations for GCN
fn fast_sqrt(x: f32) -> f32 {
    var bits = bitcast<u32>(x);
        bits = bits >> 1u;
        bits = bits + 0x1fbd1df5u;
    return bitcast<f32>(bits);
}

fn fast_length_2d(a: vec2<f32>) -> f32 {
    return fast_sqrt(a.x * a.x + a.y * a.y);
}

@fragment
fn fragment(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    var totalForce: vec2<f32> = vec2<f32>(0.0, 0.0);
    let pos = fragCoord.xy; // Assuming fragCoord.xy gives us the position in some space where we calculate forces

    // Calculate the gravitational force from each body
    for (var i = 0u; i < arrayLength(&positions); i++) {
        let force = computeGravitationalVector(pos, positions[i], masses[i]);
        totalForce += force;
    }

    // Calculate the magnitude of the total gravitational force
    let forceMagnitude = fast_length_2d(totalForce);
    let maxMagnitude = fast_length_2d(computeGravitationalVector(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0), parameters.max_mass));

    // Discretize this magnitude to create contour lines
    let colorIntensity = forceMagnitude / maxMagnitude;

    // Color mapping based on contour level
    // Alpha should be the intensity
    let color = vec4<f32>(1.0, 1.0, 1.0, colorIntensity);

    return color;
}