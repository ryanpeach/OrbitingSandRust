//! This module contains the components for the heat simulation.
//! Mostly relates to units of measure and the conversion between them.

use std::time::Duration;

use bevy::{ecs::component::Component, render::color::Color};
use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};
use ndarray::Array2;

use crate::physics::orbits::{components::Mass, nbody::Force};

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

/// The density of the element relative to the cell width
/// In units of $ \frac{kg}{m^2} $
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, Sub)]
pub struct Density(pub f32);

impl Density {
    /// This gets the mass of the element based on the cell_width
    pub fn mass(&self, cell_width: Length) -> Mass {
        Mass(self.0 * cell_width.area().0)
    }

    /// This gets the mass of the element based on the cell_width in matrix form
    pub fn matrix_mass(density_matrix: &Array2<f32>, cell_width: Length) -> Array2<f32> {
        density_matrix * cell_width.area().0
    }
}

/// # Compressibility
///
/// So in a planet what is the density of a material under pressure?
///
/// Well in a simplistic form, we are going to assume its some constant "normal"
/// density at ATP, and then we are going to linearly increase it based on pressure.
///
/// $ d_{1} = d_{0} + c * p * m_{a} $
///
/// where
///
/// * $d_{1}$ is the new density $\frac{kg}{m^2}$
/// * $d_{0}$ is the old density $\frac{kg}{m^2}$
/// * $m_{a}$ is the mass above the chunk in $kg$
/// * $c$ is the compressability $\frac{kg}{m^2 N}$
/// * $p$ is a conversion factor $\frac{N}{kg}$
///
/// This is a simplification, but it should be good enough for now.
///
/// We will also assume that the mass above the chunk is perportional to pressure,
/// by applying some conversion constant $p$.
///
/// Lastly, to speed up the calculation, we will apply the same pressure
/// uniformly to the whole chunk from the total mass of the chunks above it, rather
/// than calculating the pressure at each point.
///
/// All of this simplifies the density to be a constant times the mass above the chunk.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, Sub)]
pub struct Compressability(pub f32);

impl Compressability {
    /// This perportions the density of the element based on the mass above it
    /// $ \frac{N}{kg} $
    const PERPORTIONALITY_CONSTANT: f32 = 1.0;

    /// This gets the density of the element based on the force applied to it
    pub fn get_density(&self, original_density: Density, force: Force) -> Density {
        original_density + Density(force.0 * self.0)
    }
    /// This gets the density of the element based on the force applied to it
    /// by proxy of using the mass of the elements above it and a perportionality constant
    /// Very much an approximation, but much faster for the simulation
    pub fn get_density_from_mass(&self, original_density: Density, mass_above: Mass) -> Density {
        original_density + Density(mass_above.0 * self.0 * Self::PERPORTIONALITY_CONSTANT)
    }

    /// This is a matrix equivalent of the get_density_from_mass function
    pub fn matrix_get_density_from_mass(
        compressability_matrix: &Array2<f32>,
        original_density: &Array2<f32>,
        mass_above: Mass,
    ) -> Array2<f32> {
        original_density + mass_above.0 * compressability_matrix * Self::PERPORTIONALITY_CONSTANT
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
/// Measured in joules per degree celsius $ J / C\degree $
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
/// Measured in joules $ J $
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
/// Measured in Kelvin $ K\degree $
/// This should be represented in "real" units, physically similar to the real world.
#[derive(
    Component, Default, Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd,
)]
pub struct ThermodynamicTemperature(pub f32);

impl ThermodynamicTemperature {
    /// The minimum color alpha for the visualization of the temperature.
    const MIN_RED: f32 = 0.0;
    /// The maximum color alpha for the visualization of the temperature.
    const MAX_RED: f32 = 1.0;

    /// Returns the color of the system based on its temperature using a logarithmic scale.
    pub fn log_color(
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

    /// Returns the color of the system based on its temperature using a linear scale.
    pub fn linear_color(
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

        // Calculate the normalized linear position of the system's temperature
        let normalized_linear_pos = (self.0 - min_temp.0) / (max_temp.0 - min_temp.0);

        // Interpolate the red value linearly between MIN_RED and MAX_RED
        let red = Self::MIN_RED + (Self::MAX_RED - Self::MIN_RED) * normalized_linear_pos;

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
