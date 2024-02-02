#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{ecs::component::Component, math::Vec2};
use derive_more::{Add, AddAssign, Sub, SubAssign};

#[derive(Component, Debug, Clone, Copy)]
pub struct GravitationalField;

#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct Mass(pub f32);

#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct Velocity(pub Vec2);
