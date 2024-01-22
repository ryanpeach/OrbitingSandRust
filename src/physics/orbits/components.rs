use bevy::{ecs::component::Component, math::Vec2};

#[derive(Component, Debug, Clone, Copy)]
pub struct GravitationalField;

#[derive(Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);
