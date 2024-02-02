use bevy::math::{Vec2, Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

/// A light source that emits light in all directions from a single point.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightSource {
    center: Vec2,
    intensity: f32,
    color: Vec3,
    falloff: Vec3,
}

/// An occluder in the shape of a circle.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightOccluder {
    center: Vec2,
    radius: f32,
}
