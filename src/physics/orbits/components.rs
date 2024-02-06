#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{ecs::component::Component, math::Vec2};
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

/// Indicates that an entity emits a gravitational field.
#[derive(Component, Debug, Clone, Copy)]
pub struct GravitationalField;

/// The mass of an entity in kilograms.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign, Sum)]
pub struct Mass(pub f32);

/// The velocity of an entity in meters per second.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct Velocity(pub Vec2);
