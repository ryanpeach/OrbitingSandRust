#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::ecs::component::Component;
use derive_more::{Add, AddAssign, Sub, SubAssign};

#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct HeatCapacity(pub f32);

#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct HeatEnergy(pub f32);

#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct ThermodynamicTemperature(pub f32);
