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

use std::time::Duration;

use bevy::{log::error, log::warn};
use ndarray::{s, Array1, Array2};
use ndarray_conv::*;

use crate::physics::{
    fallingsand::{
        convolution::behaviors::ElementGridConvolutionNeighborTemperatures,
        data::element_grid::ElementGrid,
        elements::element::{Element, ElementType},
        mesh::chunk_coords::{ChunkCoords, PartialLayerChunkCoordsBuilder},
        util::vectors::{ChunkIjkVector, JkVector},
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
        let temperature = Array2::from_elem((width + 2, height + 2), 0.0);
        let thermal_conductivity = Array2::from_elem((width, height), 0.0);
        let specific_heat_capacity = Array2::from_elem((width, height), 0.0);
        let density = Array2::from_elem((width, height), 0.0);
        let compressability = Array2::from_elem((width, height), 0.0);
        Self {
            cell_width,
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
            delta_multiplier: 1.0,
            enable_compression: true,
            // This will leak a little heat to space over time
            top_temp_mult: 1.0,
            compressability,
            total_mass_above: Mass(-1.0),
        }
    }

    /// Add an element to the heat propogation system
    #[allow(clippy::borrowed_box)]
    pub fn add(&mut self, coords: &ChunkCoords, jk_vector: JkVector, elem: &Box<dyn Element>) {
        let density = elem.get_density();
        let specific_heat = elem.get_specific_heat();
        let mass = density.mass(self.cell_width);
        let heat_capacity = specific_heat.heat_capacity(mass);
        let idx: [usize; 2] = jk_vector.to_ndarray_coords(coords).into();
        let one_plus_idx: [usize; 2] = [idx[0] + 1, idx[1] + 1];
        self.temperature[one_plus_idx] = elem.get_heat().temperature(heat_capacity).0;
        self.thermal_conductivity[idx] = elem.get_thermal_conductivity().0;
        self.specific_heat_capacity[idx] = specific_heat.0;
        self.density[idx] = density.0;
        self.compressability[idx] = elem.get_compressability().0;
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
        // Remember ndarrays are row-major and the LT is 0,0
        // so we are going to make some named slices to make this easier
        let left_side = s![0, 1..-1];
        let right_side = s![-1, 1..-1];
        let top_side = s![1..-1, 0];
        let second_to_top_side = s![1..-1, 1];
        let bottom_side = s![1..-1, -1];
        let second_to_bottom_side = s![1..-1, -2];

        // Set the border temperatures
        self.temperature
            .slice_mut(left_side)
            .assign(&neighbor_temperatures.left);
        self.temperature
            .slice_mut(right_side)
            .assign(&neighbor_temperatures.right);
        if let Some(top) = neighbor_temperatures.top {
            self.temperature.slice_mut(top_side).assign(&top);
        } else {
            // Else the top is open to space
            // and it will be the same as the next layer down times some multiplier
            let second_last_row =
                self.temperature.slice(second_to_top_side).to_owned() * self.top_temp_mult;
            self.temperature
                .slice_mut(top_side)
                .assign(&second_last_row);
        }
        if let Some(bottom) = neighbor_temperatures.bottom {
            self.temperature.slice_mut(bottom_side).assign(&bottom);
        } else {
            // Else the bottom is the bottom of the world
            // so we will set it to the same temp as the next layer up
            let second_row = self.temperature.slice(second_to_bottom_side).to_owned();
            self.temperature.slice_mut(bottom_side).assign(&second_row);
        }

        // Now we just need to interpolate the corners
        let dim = self.temperature.dim();
        self.temperature[[0, 0]] = (self.temperature[[0, 1]] + self.temperature[[1, 0]]) / 2.0;
        self.temperature[[0, dim.1 - 1]] =
            (self.temperature[[0, dim.1 - 2]] + self.temperature[[1, dim.1 - 1]]) / 2.0;
        self.temperature[[dim.0 - 1, dim.1 - 1]] = (self.temperature[[dim.0 - 1, dim.1 - 2]]
            + self.temperature[[dim.0 - 2, dim.1 - 1]])
            / 2.0;
        self.temperature[[dim.0 - 1, 0]] =
            (self.temperature[[dim.0 - 1, 1]] + self.temperature[[dim.0 - 2, 0]]) / 2.0;
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
        // Apparently it's VERY important that the center be a negative number
        let laplace_kernel = Array2::from_shape_vec(
            (3, 3),
            vec![
                1., 1., 1., //
                1., -8., 1., //
                1., 1., 1., //
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
        if !new_heat_energy.iter().all(|&x| x.is_finite()) {
            error!("New heat energy is not finite");
            return;
        }

        // Save the new temperature
        self.temperature
            .slice_mut(s![1..-1, 1..-1])
            .assign(&new_temp);
    }

    /// Get the temperature array
    pub fn get_temperature(&self) -> &Array2<f32> {
        &self.temperature
    }

    /// Apply the new heat energy grid to the elements
    pub fn apply_to_grid(&self, chunk: &mut ElementGrid, current_time: Clock) {
        let coords = *chunk.get_chunk_coords();
        for k in 0..self.temperature.dim().0 - 2 {
            for j in 0..self.temperature.dim().1 - 2 {
                let idx = JkVector::new(j, k);
                let elem = chunk.get_mut(idx);
                if elem.get_specific_heat().0 == 0.0 {
                    continue;
                }
                let idx: [usize; 2] = idx.to_ndarray_coords(&coords).into();
                let one_plus_idx: [usize; 2] = [idx[0] + 1, idx[1] + 1];
                elem.set_heat(HeatEnergy(self.temperature[one_plus_idx]), current_time)
                    .unwrap();
            }
        }
    }
}

/// # Testing
/// These are some helpful heat based sanity testing functions
/// for each element to test their heat properties
impl PropogateHeat {
    /// Surrounded by 0.0 temperature, the heat average over a 5x5 grid
    /// should disipate to half its original temperature in exactly `frames` frames
    pub fn test_heat_disipation_rate_in_space(
        frames: u32,
        frame_rate: u32,
        element_type: ElementType,
    ) {
        // Set up the chunk coords
        let coords = PartialLayerChunkCoordsBuilder::new()
            .num_concentric_circles(5)
            .start_radial_line(0)
            .end_radial_line(5)
            .layer_num_radial_lines(5)
            .start_concentric_circle_layer_relative(0)
            .cell_radius(Length(1.0))
            .chunk_idx(ChunkIjkVector::new(0, 0, 0))
            .start_concentric_circle_absolute(0)
            .build();

        // Set up the builder
        let mut builder = PropogateHeatBuilder::new(5, 5, Length(1.0));
        builder.enable_compression(false);
        builder.total_mass_above(Mass(0.0));
        for j in 0..5 {
            for k in 0..5 {
                builder.add(
                    &coords,
                    JkVector::new(j, k),
                    &element_type.get_element(Length(1.0)).box_clone(),
                );
            }
        }

        // This is the border
        let mut heat = builder.build(ElementGridConvolutionNeighborTemperatures {
            left: Array1::zeros(5),
            right: Array1::zeros(5),
            top: Some(Array1::zeros(5)),
            bottom: Some(Array1::zeros(5)),
        });

        let mut clock = Clock::default();
        let avg = heat.get_temperature().sum() / (5 * 5) as f32;
        for frame_cnt in 0..frames {
            assert!(
                (heat.get_temperature().sum() / (5 * 5) as f32) < (avg / 2.0),
                "Took less than {} frames to cool down: {}",
                frames,
                frame_cnt
            );

            // Update the clock
            clock.update(Duration::from_secs_f32(1.0 / frame_rate as f32));

            // Check that the heat is not yet near zero in the center
            let heat_energy = heat.get_temperature().clone();
            if frame_cnt % frame_rate == 0 {
                println!("#{:?} Heat energy:\n{:?}", frame_cnt, heat_energy);
            }

            // Propogate the heat
            heat.propagate_heat(clock);
        }

        // Check that the heat is near zero in the center
        assert!(
            (heat.get_temperature().sum() / (5 * 5) as f32) < (avg / 2.0),
            "Took longer than {} frames to cool down.",
            frames
        );
    }
}
