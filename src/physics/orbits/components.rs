//! Contains the components used in the orbit system
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use bevy::{
    ecs::{component::Component, system::Query},
    math::Vec2,
    render::extract_component::ExtractComponent,
    transform::components::{GlobalTransform},
};

/// Indicates that this entity emits a gravitational field
#[derive(Component, ExtractComponent, Debug, Clone, Copy)]
pub struct GravitationalField;

/// The mass of an entity
#[derive(Component, ExtractComponent, Debug, Clone, Copy)]
pub struct Mass(pub f32);

/// The velocity of an entity
#[derive(Component, ExtractComponent, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

/// Used as an extractable type of Transform
/// You shouldnt need to modify this type from outside of the following system
#[derive(Component, ExtractComponent, Default, Debug, Clone, Copy)]
pub struct OrbitalPosition(Vec2);

impl OrbitalPosition {
    /// Keeps Orbital Position tracking the Global Position
    pub fn follow_transform_system(mut query: Query<(&mut OrbitalPosition, &GlobalTransform)>) {
        query.par_iter_mut().for_each(|(mut pos, transform)| {
            pos.0 = transform.translation().truncate();
        });
    }

    /// Gets the position of the entity
    pub fn position(&self) -> Vec2 {
        self.0
    }
}
