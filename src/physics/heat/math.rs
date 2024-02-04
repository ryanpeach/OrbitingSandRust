//! Heat propagation math.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{log::trace, time::Time, ui::debug};
use ndarray::{s, Array2};
use ndarray_conv::*;

use crate::physics::{
    fallingsand::{
        convolution::{
            behaviors::ElementGridConvolutionNeighborTemperatures,
            neighbor_grids::ElementGridConvolutionNeighborGrids,
        },
        data::element_grid::ElementGrid,
        elements::element::{Compressability, Density, Element},
        util::vectors::JkVector,
    },
    heat::components::{HeatCapacity, SpecificHeat},
    orbits::components::Mass,
    util::clock::Clock,
};

use super::components::{HeatEnergy, Length, ThermodynamicTemperature};

pub struct PropogateHeatBuilder {
    cell_width: Length,
    temperature: Array2<f32>,
    thermal_conductivity: Array2<f32>,
    specific_heat_capacity: Array2<f32>,
    density: Array2<f32>,
    compressability: Array2<f32>,
    time: Clock,
    total_mass_above: Mass,
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
        let time = Clock::default();
        Self {
            cell_width,
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
            compressability,
            time,
            // To be set later
            total_mass_above: Mass(-1.0),
        }
    }

    /// Add an element to the heat propogation system
    pub fn add(&mut self, jk_vector: JkVector, elem: &Box<dyn Element>) {
        let density = elem.get_density();
        let specific_heat = elem.get_specific_heat();
        let mass = density.mass(self.cell_width);
        let heat_capacity = specific_heat.heat_capacity(mass);
        self.temperature[[jk_vector.j as usize + 1, jk_vector.k as usize + 1]] =
            elem.get_heat().temperature(heat_capacity).0;
        self.thermal_conductivity[[jk_vector.j as usize, jk_vector.k as usize]] =
            elem.get_thermal_conductivity().0;
        self.specific_heat_capacity[[jk_vector.j as usize, jk_vector.k as usize]] = specific_heat.0;
        self.density[[jk_vector.j as usize, jk_vector.k as usize]] = density.0;
        self.compressability[[jk_vector.j as usize, jk_vector.k as usize]] =
            elem.get_compressability().0;
    }

    /// Simple setter for the total mass above the chunk
    pub fn total_mass_above(&mut self, total_mass_above: Mass) {
        self.total_mass_above = total_mass_above;
    }

    /// Set the temperature of the border cells based on the convolved neighbor temperatures
    pub fn border_temperatures(
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
            self.temperature.slice_mut(s![0, ..]).fill(top.0);
            self.temperature
                .slice_mut(s![0, 0])
                .fill((top.0 + neighbor_temperatures.left.0) / 2.0);
            self.temperature
                .slice_mut(s![0, -1])
                .fill((top.0 + neighbor_temperatures.right.0) / 2.0);
        }
        if let Some(bottom) = neighbor_temperatures.bottom {
            self.temperature.slice_mut(s![-1, ..]).fill(bottom.0);
            self.temperature
                .slice_mut(s![-1, 0])
                .fill((bottom.0 + neighbor_temperatures.left.0) / 2.0);
            self.temperature
                .slice_mut(s![-1, -1])
                .fill((bottom.0 + neighbor_temperatures.right.0) / 2.0);
        }
    }

    pub fn build(self) -> PropogateHeat {
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
        PropogateHeat {
            cell_width: self.cell_width,
            temperature: self.temperature,
            total_mass_above: self.total_mass_above,
            thermal_conductivity: self.thermal_conductivity,
            specific_heat_capacity: self.specific_heat_capacity,
            density: self.density,
            compressability: self.compressability,
            time: self.time,
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
    /// The time since last processed
    time: Clock,
}

impl PropogateHeat {
    /// Propogate the heat in the grid
    pub fn propagate_heat(&self, element_grid: &mut ElementGrid) {
        // Define the convolution kernel
        let laplace_kernel = Array2::from_shape_vec(
            (3, 3),
            vec![
                0.125, 0.125, 0.125, //
                0.125, -1.0, 0.125, //
                0.125, 0.125, 0.125, //
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
                PaddingMode::Reflect, // Doesn't matter in Valid mode
            )
            .unwrap();

        // Get the alpha grid
        let alpha = &self.thermal_conductivity
            / (&self.specific_heat_capacity
                * Compressability::matrix_get_density_from_mass(
                    &self.compressability,
                    &self.density,
                    self.total_mass_above,
                ));
        // Replace all Nans with zero because anything that has specific heat capacity 0 also has 0 thermal conductivity
        let alpha = alpha.mapv(|x| if x.is_finite() { x } else { 0.0 });
        let delta_temperature =
            alpha * second_gradient_temperature * self.time.get_last_delta().as_secs_f32();

        // Check everything is finite
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

        // Apply the new heat energy to the elements
        self.apply(element_grid, new_heat_energy);
    }

    /// Apply the new heat energy grid to the elements
    fn apply(&self, chunk: &mut ElementGrid, new_heat_energy: Array2<f32>) {
        for j in 0..self.temperature.dim().0 - 2 {
            for k in 0..self.temperature.dim().1 - 2 {
                let elem = chunk.get_mut(JkVector::new(j, k));
                if elem.get_specific_heat().0 == 0.0 {
                    continue;
                }
                elem.set_heat(HeatEnergy(new_heat_energy[[j, k]])).unwrap();
            }
        }
    }
}
