#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{ecs::component::Component, render::color::Color};
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// The amount of heat energy required to raise the temperature of a system by one degree.
/// Measured in joules per Kelvin.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct HeatCapacity(pub f32);

/// The amount of heat energy in a system.
/// Measured in joules.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct HeatEnergy(pub f32);

impl HeatEnergy {
    /// Returns the temperature of the system.
    pub fn temperature(&self, heat_capacity: HeatCapacity) -> ThermodynamicTemperature {
        ThermodynamicTemperature(self.0 / heat_capacity.0)
    }
}

/// The temperature of a system.
/// Measured in Kelvin.
#[derive(
    Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd,
)]
pub struct ThermodynamicTemperature(pub f32);

impl ThermodynamicTemperature {
    /// Returns the color of the system.
    pub fn color(&self, max_temp: ThermodynamicTemperature) -> Color {
        let red = 1.0;
        let alpha = self.0 / max_temp.0;
        Color::rgba(red, 0.0, 0.0, alpha)
    }
}
