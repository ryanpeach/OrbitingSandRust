//! Heat propagation math.
//! # Heat
//!
//! <https://en.wikipedia.org/wiki/Thermal_diffusivity>
//!
//! The heat propogation system is based on the heat equation:
//!
//! $ \alpha = \frac{ k }{ \rho c_{p} } $
//!
//! $ \frac{\partial T}{\partial t} = \alpha \nabla^2 T $
//!
//! where:
//! * $k$ is thermal conductivity in $\frac{W}{m K}$
//! * $c_{p}$ is specific heat capacity in $\frac{J}{kg K}$
//! * $p$ is density in $\frac{kg}{m^2}$
//! * $T$ is temperature in $K\degree$
//!
//! This basically tells us that the time derivative of the temperature is equal to
//! the second gradient of the temperature times a constant relating the density
//! and the heat properties of the material.
//!
//! # Laplace Kernel
//!
//! <https://homepages.inf.ed.ac.uk/rbf/HIPR2/log.htm>
//!
//! The laplace kernel is a 3x3 kernel that looks like this:
//!
//! $ \begin{bmatrix} -1 & -1 & -1 \\\\ -1 & 8 & -1 \\\\ -1 & -1 & -1 \end{bmatrix} $
//!
//! It represents the second gradient of a matrix.
//! If we represent the temperature as a matrix, then the second gradient of the temperature
//! is the convolution of the temperature with the laplace kernel.
//!
//! It can be quickly calculated using ndarray-conv using the fft method. This also
//! uses the matrix operators on your cpu rather than using loops, making it very fast.

use bevy::log::warn;
use ndarray::{s, Array2};
use ndarray_conv::*;

use crate::physics::{
    fallingsand::{
        convolution::behaviors::ElementGridConvolutionNeighborTemperatures,
        data::element_grid::ElementGrid, elements::element::Element, util::vectors::JkVector,
    },
    heat::components::{Compressability, Density, SpecificHeat},
    orbits::components::Mass,
    util::clock::Clock,
};

use super::components::{HeatEnergy, Length, ThermodynamicTemperature};

/// The builder of the inputs to the heat propogation system
pub struct PropogateHeatBuilder {
    /// The width of each cell
    cell_width: Length,
    /// The temperature of each cell in the chunk
    temperature: Array2<f32>,
    /// The thermal conductivity of each cell in the chunk
    thermal_conductivity: Array2<f32>,
    /// The specific heat capacity of each cell in the chunk
    specific_heat_capacity: Array2<f32>,
    /// The density of each cell in the chunk
    density: Array2<f32>,
    /// The temperature "of space" relative to the temperature of the top layer
    /// Set this to 0 to have a full temperature gradient to space (space will feel like 0deg)
    /// Set this to 1 to have no temperature gradient to space (space will feel like the top layer)
    /// TODO: I don't know what the thermal conductivity to space will be though
    top_temp_mult: f32,
    /// The compressability of each cell in the chunk
    compressability: Array2<f32>,
    /// The total mass above the chunk
    total_mass_above: Mass,
    /// The multiplier for the delta temperature
    /// Greater than 1 speeds up propogation
    /// Greater than 0 but less than 1 slows down propogation
    /// Must be greater than 0 and finite
    /// Defaults to 1
    delta_multiplier: f32,
    /// Whether to enable compression
    /// Defaults to true
    enable_compression: bool,
}

impl PropogateHeatBuilder {
    /// Create a new heat propogation system with the given width and height
    /// All the arrays will be initialized to 0
    pub fn new(height: usize, width: usize, cell_width: Length) -> Self {
        let temperature = Array2::from_elem((height + 2, width + 2), 0.0);
        let thermal_conductivity = Array2::from_elem((height, width), 0.0);
        let specific_heat_capacity = Array2::from_elem((height, width), 0.0);
        let density = Array2::from_elem((height, width), 0.0);
        let compressability = Array2::from_elem((height, width), 0.0);
        Self {
            cell_width,
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
            delta_multiplier: 1.0,
            enable_compression: true,
            // This will leak a little heat to space over time
            top_temp_mult: 0.99,
            compressability,
            total_mass_above: Mass(-1.0),
        }
    }

    /// Add an element to the heat propogation system
    #[allow(clippy::borrowed_box)]
    pub fn add(&mut self, jk_vector: JkVector, elem: &Box<dyn Element>) {
        let density = elem.get_density();
        let specific_heat = elem.get_specific_heat();
        let mass = density.mass(self.cell_width);
        let heat_capacity = specific_heat.heat_capacity(mass);
        self.temperature[[jk_vector.j + 1, jk_vector.k + 1]] =
            elem.get_heat().temperature(heat_capacity).0;
        self.thermal_conductivity[[jk_vector.j, jk_vector.k]] = elem.get_thermal_conductivity().0;
        self.specific_heat_capacity[[jk_vector.j, jk_vector.k]] = specific_heat.0;
        self.density[[jk_vector.j, jk_vector.k]] = density.0;
        self.compressability[[jk_vector.j, jk_vector.k]] = elem.get_compressability().0;
    }

    /// Simple setter for the total mass above the chunk
    pub fn total_mass_above(&mut self, total_mass_above: Mass) {
        self.total_mass_above = total_mass_above;
    }

    /// Set the temperature "of space"
    /// If you don't want to set it, just don't call this method
    pub fn top_temp_mult(&mut self, top_temp_mult: f32) {
        self.top_temp_mult = top_temp_mult
    }

    /// Set the multiplier for the delta temperature
    pub fn delta_multiplier(&mut self, delta_multiplier: f32) {
        self.delta_multiplier = delta_multiplier;
    }

    /// Set whether to enable compression
    pub fn enable_compression(&mut self, enable_compression: bool) {
        self.enable_compression = enable_compression;
    }

    /// Set the temperature of the border cells based on the convolved neighbor temperatures
    /// This is only called by the build method because it needs to be called after all
    /// adds are done
    fn border_temperatures(
        &mut self,
        neighbor_temperatures: ElementGridConvolutionNeighborTemperatures,
    ) {
        self.temperature
            .slice_mut(s![.., 0])
            .fill(neighbor_temperatures.left.0);
        self.temperature
            .slice_mut(s![.., -1])
            .fill(neighbor_temperatures.right.0);
        if let Some(top) = neighbor_temperatures.top {
            self.temperature.slice_mut(s![-1, ..]).fill(top.0);
            self.temperature
                .slice_mut(s![-1, 0])
                .fill((top.0 + neighbor_temperatures.left.0) / 2.0);
            self.temperature
                .slice_mut(s![-1, -1])
                .fill((top.0 + neighbor_temperatures.right.0) / 2.0);
        } else {
            // Else the top is open to the atmosphere and thus the temperature is Some(top_temp)
            // Or you can give it None and it will be the same as the next layer down
            // This would model no heat loss to space
            let second_last_row =
                self.temperature.slice(s![-2, ..]).to_owned() * self.top_temp_mult;
            self.temperature
                .slice_mut(s![-1, ..])
                .assign(&second_last_row);
        }
        if let Some(bottom) = neighbor_temperatures.bottom {
            self.temperature.slice_mut(s![0, ..]).fill(bottom.0);
            self.temperature
                .slice_mut(s![0, 0])
                .fill((bottom.0 + neighbor_temperatures.left.0) / 2.0);
            self.temperature
                .slice_mut(s![0, -1])
                .fill((bottom.0 + neighbor_temperatures.right.0) / 2.0);
        } else {
            // Else the bottom is the bottom of the world
            // so we will set it to the same temp as the next layer up
            let second_row = self.temperature.slice(s![1, ..]).to_owned();
            self.temperature.slice_mut(s![0, ..]).assign(&second_row);
        }
    }

    /// Create the structure and test all the values
    pub fn build(
        mut self,
        neighbor_temperatures: ElementGridConvolutionNeighborTemperatures,
    ) -> PropogateHeat {
        // Set the border temperatures
        self.border_temperatures(neighbor_temperatures);
        // Check you called the methods
        debug_assert!(
            self.total_mass_above.0 >= 0.0,
            "Total mass above must be greater than or equal to 0. Did you set it?"
        );
        // Check everything is the right size
        debug_assert_eq!(
            self.thermal_conductivity.dim(),
            self.specific_heat_capacity.dim(),
            "Thermal conductivity and specific heat capacity must be the same size"
        );
        debug_assert_eq!(
            self.thermal_conductivity.dim(),
            self.density.dim(),
            "Thermal conductivity and density must be the same size"
        );
        debug_assert_eq!(
            self.temperature.dim().0,
            self.thermal_conductivity.dim().0 + 2,
            "Temperature must be the size of the thermal conductivity + 4 on both hight and width"
        );
        debug_assert_eq!(
            self.temperature.dim().1,
            self.thermal_conductivity.dim().1 + 2,
            "Temperature must be the size of the thermal conductivity + 4 on both hight and width"
        );
        debug_assert_eq!(
            self.compressability.dim(),
            self.thermal_conductivity.dim(),
            "Compressability must be the same size as the thermal conductivity"
        );
        debug_assert!(
            self.cell_width.0 >= 0.0,
            "Cell width must be greater than 0"
        );
        debug_assert!(
            self.total_mass_above.0 >= 0.0,
            "Total mass above must be greater than or equal to 0. Did you set it?"
        );
        // Check everything is finite
        debug_assert!(
            self.temperature.iter().all(|&x| x.is_finite()),
            "Temperature must be finite"
        );
        debug_assert!(
            self.thermal_conductivity.iter().all(|&x| x.is_finite()),
            "Thermal conductivity must be finite"
        );
        debug_assert!(
            self.specific_heat_capacity.iter().all(|&x| x.is_finite()),
            "Specific heat capacity must be finite"
        );
        debug_assert!(
            self.density.iter().all(|&x| x.is_finite()),
            "Density must be finite"
        );
        debug_assert!(
            self.compressability.iter().all(|&x| x.is_finite()),
            "Compressability must be finite"
        );
        debug_assert!(
            self.delta_multiplier.is_finite(),
            "Delta multiplier must be finite"
        );
        debug_assert!(
            self.delta_multiplier > 0.0,
            "Delta multiplier must be greater than 0"
        );
        PropogateHeat {
            cell_width: self.cell_width,
            temperature: self.temperature,
            total_mass_above: self.total_mass_above,
            thermal_conductivity: self.thermal_conductivity,
            specific_heat_capacity: self.specific_heat_capacity,
            density: self.density,
            compressability: self.compressability,
            delta_multiplier: self.delta_multiplier,
            enable_compression: self.enable_compression,
        }
    }
}
/// The inputs to the heat propogation system
pub struct PropogateHeat {
    /// The width of each cell
    cell_width: Length,
    /// The temperature of each cell in the chunk
    temperature: Array2<f32>,
    /// The total mass above the chunk
    total_mass_above: Mass,
    /// The thermal conductivity of each cell in the chunk
    /// Should be the size of the chunk
    thermal_conductivity: Array2<f32>,
    /// The specific heat capacity of each cell in the chunk
    /// Should be the size of the chunk
    specific_heat_capacity: Array2<f32>,
    /// The density of each cell in the chunk
    /// Should be the size of the chunk
    density: Array2<f32>,
    /// Compressability of each cell in the chunk
    /// Should be the size of the chunk
    compressability: Array2<f32>,
    /// Whether to enable compression
    enable_compression: bool,
    /// The multiplier for the delta temperature
    /// Greater than 1 speeds up propogation
    /// Greater than 0 but less than 1 slows down propogation
    /// Must be greater than 0 and finite
    /// Defaults to 1
    delta_multiplier: f32,
}

impl PropogateHeat {
    /// This is the main method of the heat propogation system
    /// Propogate the heat one iteration
    /// Rerun this method multiple times to propogate the heat multiple iterations
    /// without needing to reinitialize the system
    /// however, movement will not be accounted for if you do this
    #[allow(clippy::reversed_empty_ranges)] // REF: https://github.com/rust-lang/rust-clippy/issues/5808
    pub fn propagate_heat(&mut self, current_time: Clock) {
        if current_time.get_last_delta().as_secs_f32() == 0.0 {
            warn!("Delta time is 0, not processing heat. May just be the first frame.");
            return;
        }

        // Define the convolution kernel
        let laplace_kernel = Array2::from_shape_vec(
            (3, 3),
            vec![
                -1., -1., -1., //
                -1., 8., -1., //
                -1., -1., -1., //
            ],
        )
        .unwrap();
        debug_assert_eq!(laplace_kernel.sum(), 0.0, "Kernel must sum to 0");

        // Convolve the temperature with the kernel to get the gradient
        let second_gradient_temperature = self
            .temperature
            .conv_2d_fft(
                &laplace_kernel,
                PaddingSize::Valid,
                PaddingMode::Zeros, // Doesn't matter in Valid mode
            )
            .unwrap();
        // trace!("Second gradient temperature sum: {}", second_gradient_temperature.sum());

        // Get the alpha grid
        // trace!("Thermal conductivity: {}", self.thermal_conductivity.sum());
        // trace!("Specific heat capacity: {}", self.specific_heat_capacity.sum());
        let density = {
            if self.enable_compression {
                Compressability::matrix_get_density_from_mass(
                    &self.compressability,
                    &self.density,
                    self.total_mass_above,
                )
            } else {
                self.density.clone()
            }
        };
        // trace!("Density: {}", matrix_get_density_from_mass.sum());
        let alpha = &self.thermal_conductivity / (&self.specific_heat_capacity * density);
        // Replace all Nans with zero because anything that has specific heat capacity 0 also has 0 thermal conductivity
        let alpha = alpha.mapv(|x| if x.is_finite() { x } else { 0.0 });
        // trace!("Apha sum: {}", alpha.sum());
        let delta_temperature = alpha
            * second_gradient_temperature
            * current_time.get_last_delta().as_secs_f32()
            * self.delta_multiplier;

        // Check everything is finite
        // trace!("Delta temperature sum: {:?}", delta_temperature.sum());
        // trace!("time: {:?}", current_time.get_last_delta().as_secs_f32());
        debug_assert!(
            delta_temperature.iter().all(|&x| x.is_finite()),
            "Delta temperature must be finite"
        );

        // calculate the new temperature
        let new_temp = &self.temperature.slice(s![1..-1, 1..-1]) + &delta_temperature;

        // Convert to heat
        let new_heat_energy = ThermodynamicTemperature::matrix_heat_energy(
            &new_temp,
            &SpecificHeat::matrix_heat_capacity(
                &self.specific_heat_capacity,
                &Density::matrix_mass(&self.density, self.cell_width),
            ),
        );

        // Check everything is finite
        debug_assert!(
            new_heat_energy.iter().all(|&x| x.is_finite()),
            "New heat energy must be finite"
        );

        // Save the new temperature
        self.temperature = new_temp;
    }

    /// Get the temperature array
    pub fn get_temperature(&self) -> &Array2<f32> {
        &self.temperature
    }

    /// Apply the new heat energy grid to the elements
    pub fn apply_to_grid(&self, chunk: &mut ElementGrid, current_time: Clock) {
        for j in 0..self.temperature.dim().0 - 2 {
            for k in 0..self.temperature.dim().1 - 2 {
                let elem = chunk.get_mut(JkVector::new(j, k));
                if elem.get_specific_heat().0 == 0.0 {
                    continue;
                }
                elem.set_heat(HeatEnergy(self.temperature[[j, k]]), current_time)
                    .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::physics::fallingsand::{
        elements::{element::ElementType, water::Water},
        mesh::chunk_coords,
    };

    use super::*;

    const CELL_WIDTH: Length = Length(1.0);

    #[test]
    fn test_sink_diffuses_to_zero() {
        // Set up the builder
        let mut builder = PropogateHeatBuilder::new(5, 5, CELL_WIDTH);
        for j in 0..5 {
            for k in 0..5 {
                builder.add(
                    JkVector::new(j, k),
                    &ElementType::Water.get_element(CELL_WIDTH).box_clone(),
                );
            }
        }
        let mut heat = builder.build(ElementGridConvolutionNeighborTemperatures {
            left: ThermodynamicTemperature(0.0),
            right: ThermodynamicTemperature(0.0),
            top: Some(ThermodynamicTemperature(0.0)),
            bottom: Some(ThermodynamicTemperature(0.0)),
        });

        // Over five seconds at 30fps
        const FRAME_RATE: u32 = 30;
        let mut clock = Clock::default();
        for frame_cnt in 0..(5 * FRAME_RATE) {
            // Update the clock
            clock.update(Duration::from_secs_f32(1.0 / FRAME_RATE as f32));

            // Check that the heat is not yet near zero in the center
            let heat_energy = heat.temperature.clone();
            if frame_cnt % FRAME_RATE == 0 {
                println!(
                    "#{:?} Heat energy:\n{:?}",
                    frame_cnt / FRAME_RATE,
                    heat_energy
                );
            }

            // Propogate the heat
            heat.propagate_heat(clock);
        }

        // Check that the heat is near zero in the center
        assert!(heat.get_temperature()[[2, 2]].abs() < 0.1);
    }
}
