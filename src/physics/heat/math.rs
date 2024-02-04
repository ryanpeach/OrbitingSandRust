//! Heat propagation math.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{time::Time, ui::debug};
use ndarray::{s, Array2};
use ndarray_conv::*;

/// The inputs to the heat propogation system
struct PropogateHeat {
    /// The temperature of each cell in the chunk
    temperature: Array2<f32>,
    /// This is the average temperature of all the surrounding cells
    /// This is used for a very rough approximation of between-chunk heat transfer
    surrounding_average_temperature: f32,
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
    /// The time step
    time: Time,
}

impl PropogateHeat {
    pub fn new(
        temperature: Array2<f32>,
        surrounding_average_temperature: f32,
        thermal_conductivity: Array2<f32>,
        specific_heat_capacity: Array2<f32>,
        density: Array2<f32>,
        compressability: Array2<f32>,
        time: Time,
    ) -> Self {
        // Check everything is the right size
        debug_assert_eq!(
            thermal_conductivity.dim(),
            specific_heat_capacity.dim(),
            "Thermal conductivity and specific heat capacity must be the same size"
        );
        debug_assert_eq!(
            thermal_conductivity.dim(),
            density.dim(),
            "Thermal conductivity and density must be the same size"
        );
        debug_assert_eq!(
            temperature.dim().0,
            thermal_conductivity.dim().0 + 4,
            "Temperature must be the size of the thermal conductivity + 4 on both hight and width"
        );
        debug_assert_eq!(
            temperature.dim().1,
            thermal_conductivity.dim().1 + 4,
            "Temperature must be the size of the thermal conductivity + 4 on both hight and width"
        );
        debug_assert_eq!(
            temperature.dim().0,
            compressability.dim().0 + 4,
            "Temperature must be the size of the compressability + 4 on both hight and width"
        );
        // Check everything is finite
        debug_assert!(
            temperature.iter().all(|&x| x.is_finite()),
            "Temperature must be finite"
        );
        debug_assert!(
            thermal_conductivity.iter().all(|&x| x.is_finite()),
            "Thermal conductivity must be finite"
        );
        debug_assert!(
            specific_heat_capacity.iter().all(|&x| x.is_finite()),
            "Specific heat capacity must be finite"
        );
        debug_assert!(
            density.iter().all(|&x| x.is_finite()),
            "Density must be finite"
        );
        debug_assert!(
            compressability.iter().all(|&x| x.is_finite()),
            "Compressability must be finite"
        );
        Self {
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
            time,
            compressability,
            surrounding_average_temperature,
        }
    }

    /// Propogate the heat in the grid and return the new temperature grid
    pub fn propagate_heat(&self) -> Array2<f32> {
        // Define the convolution kernel
        let delta_kernel = Array2::from_shape_vec(
            (3, 3),
            vec![
                0.125, 0.125, 0.125, //
                0.125, -1.0, 0.125, //
                0.125, 0.125, 0.125, //
            ],
        )
        .unwrap();
        debug_assert_eq!(delta_kernel.sum(), 0.0, "Kernel must sum to 0");

        // Convolve the temperature with the kernel to get the gradient
        let gradient_temperature = self
            .temperature
            .conv_2d_fft(
                &delta_kernel,
                PaddingSize::Same,
                PaddingMode::Const(self.surrounding_average_temperature),
            )
            .unwrap();

        // Get the second order gradient
        let second_gradient_temperature = gradient_temperature
            .conv_2d_fft(&delta_kernel, PaddingSize::Same, PaddingMode::Zeros)
            .unwrap();

        // Get the alpha grid
        let alpha = &self.thermal_conductivity / (&self.specific_heat_capacity * &self.density);
        let delta_temperature = alpha * second_gradient_temperature * self.time.delta_seconds();

        // Check everything is finite
        debug_assert!(
            delta_temperature.iter().all(|&x| x.is_finite()),
            "Delta temperature must be finite"
        );

        // Return the new temperature
        &self.temperature + &delta_temperature
    }
}
