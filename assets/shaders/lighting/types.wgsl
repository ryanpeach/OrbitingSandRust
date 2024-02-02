#define_import_path orbiting_sand_rust::types

struct LightSource {
    center:    vec2<f32>,
    intensity: f32,
    color:     vec3<f32>,
    falloff:   vec3<f32>,
}

struct LightSourceBuffer {
    count: u32,
    data:  array<LightSource>,
}

struct LightOccluder {
    center: vec2<f32>,
    radius: f32,
}

struct LightOccluderBuffer {
    count: u32,
    data:  array<LightOccluder>,
}