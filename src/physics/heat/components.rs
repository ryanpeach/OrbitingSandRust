#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::time::Duration;

use bevy::{ecs::component::Component, render::color::Color};
use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};
use ndarray::Array2;

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

    /// Returns the heat capacity of the system.
    pub fn matrix_heat_capacity(specific_heat: &Array2<f32>, mass: &Array2<f32>) -> Array2<f32> {
        specific_heat * mass
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
        if self.0 == 0.0 {
            return ThermodynamicTemperature(0.0);
        }
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
        if heat_capacity.0 == 0.0 {
            return ThermodynamicTemperature(0.0);
        }
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
    const MIN_RED: f32 = 0.5;
    const MAX_RED: f32 = 1.0;

    /// Returns the color of the system.
    pub fn color(
        &self,
        max_temp: ThermodynamicTemperature,
        min_temp: ThermodynamicTemperature,
    ) -> Color {
        debug_assert_ne!(max_temp.0, 0.0, "max_temp cannot be zero");
        debug_assert_ne!(min_temp.0, 0.0, "min_temp cannot be zero");
        debug_assert!(
            max_temp.0 > min_temp.0,
            "max_temp must be greater than min_temp"
        );
        if self.0 == 0.0 {
            return Color::rgba(0.0, 0.0, 0.0, 0.0);
        }

        let min_temp_log = min_temp.0.log(10.0);
        let max_temp_log = max_temp.0.log(10.0);
        let temp_log = self.0.log(10.0);

        // Calculate the normalized logarithmic position of the system's temperature
        let normalized_log_pos = (temp_log - min_temp_log) / (max_temp_log - min_temp_log);

        // Interpolate the red value logarithmically between MIN_RED and MAX_RED
        let red = Self::MIN_RED + (Self::MAX_RED - Self::MIN_RED) * normalized_log_pos;

        Color::rgba(1.0, 0.0, 0.0, red)
    }

    /// Returns the heat energy of the system.
    pub fn heat_energy(&self, heat_capacity: HeatCapacity) -> HeatEnergy {
        HeatEnergy(self.0 * heat_capacity.0)
    }

    /// Returns the heat energy of the system.
    pub fn matrix_heat_energy(
        temperature: &Array2<f32>,
        heat_capacity: &Array2<f32>,
    ) -> Array2<f32> {
        temperature * heat_capacity
    }
}

/// The temperature in kelvin of room temperature.
pub const ROOM_TEMPERATURE_K: ThermodynamicTemperature = ThermodynamicTemperature(293.15);
