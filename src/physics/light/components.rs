//! Types for light physics.

use bevy::{ecs::component::Component, render::color::Color};

/// An occluder in the shape of a circle.
/// It should be UV'd to look like a lit sphere.
#[derive(Component, Debug, Clone)]
pub struct LightOccluder {
    /// The radius of the sphere.
    pub radius: f32,
}

/// A light source that emits light in all directions from a single point.
#[derive(Component, Debug, Clone)]
pub struct LightSource {
    /// Color of the light.
    pub color: Color,
    /// The intensity of the light.
    pub intensity: f32,
    /// The falloff of the light as a multiplier.
    pub falloff: f32,
}
