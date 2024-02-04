#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::time::Duration;

use bevy::{ecs::component::Component, render::color::Color};
use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};

use crate::physics::orbits::components::Mass;

/// The length of a system in meters.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign, From, Into)]
pub struct Length(pub f32);

impl Length {
    /// Returns the volume of the system.
    pub fn area(&self) -> Area {
        Area(self.0 * self.0)
    }
}

/// The area of a system in square meters.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct Area(pub f32);

impl Area {
    /// Returns the volume of the system.
    pub fn from_length(length: Length) -> Self {
        Area(length.0 * length.0)
    }
}

/// The amount of heat energy required to raise 1 kilogram of the substance by one degree.
/// Measured in joules per kilogram per degree celsius.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct SpecificHeat(pub f32);

impl SpecificHeat {
    /// Returns the heat capacity of the system.
    pub fn heat_capacity(&self, mass: Mass) -> HeatCapacity {
        HeatCapacity(self.0 * mass.0)
    }
}

/// Thermal Conductivity
/// The ability of a material to conduct heat.
/// Measured in watts per meter per degree celsius.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct ThermalConductivity(pub f32);

impl ThermalConductivity {
    /// Returns the heat capacity of the system.
    pub fn heat_capacity(&self, cell_width: Length, dt: Duration) -> HeatCapacity {
        HeatCapacity(self.0 * cell_width.0 * dt.as_secs_f32())
    }
}

/// The amount of heat energy required to raise the temperature of a system.
/// Measured in joules per degree celsius.
#[derive(Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct HeatCapacity(pub f32);

impl HeatCapacity {
    /// Returns the temperature of the system.
    pub fn temperature(&self, heat_energy: HeatEnergy) -> ThermodynamicTemperature {
        ThermodynamicTemperature(heat_energy.0 / self.0)
    }
}

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
/// This should be represented in "real" units, physically similar to the real world.
#[derive(
    Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd,
)]
pub struct ThermodynamicTemperature(pub f32);

impl ThermodynamicTemperature {
    /// Returns the color of the system.
    pub fn color(&self, max_temp: ThermodynamicTemperature) -> Color {
        let red = 1.0;
        debug_assert_ne!(max_temp.0, 0.0, "max_temp cannot be zero");
        let alpha = self.0 / max_temp.0;
        Color::rgba(red, 0.0, 0.0, alpha)
    }

    /// Returns the heat energy of the system.
    pub fn heat_energy(&self, heat_capacity: HeatCapacity) -> HeatEnergy {
        HeatEnergy(self.0 * heat_capacity.0)
    }
}

/// The temperature in kelvin of room temperature.
pub const ROOM_TEMPERATURE_K: ThermodynamicTemperature = ThermodynamicTemperature(293.15);
