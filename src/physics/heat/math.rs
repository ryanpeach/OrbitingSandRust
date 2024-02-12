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

use bevy::{log::error, log::warn};
use ndarray::{linalg::Dot, s, Array1, Array2, ArrayView1, Ix1, Ix2, SliceInfo, SliceInfoElem};
use rand::seq;
use sprs::{prod, CsMat, MulAcc, TriMat};
use std::time::Duration;

use crate::physics::{
    fallingsand::{
        data::element_grid::ElementGrid,
        elements::element::{Element, ElementType},
        mesh::chunk_coords::{ChunkCoords, PartialLayerChunkCoordsBuilder},
        util::vectors::{ChunkIjkVector, JkVector},
    },
    heat::{
        components::{Compressability, Density, SpecificHeat},
        convolution::MatrixBorderHeatProperties,
    },
    orbits::components::Mass,
    util::clock::Clock,
};

use super::{
    components::{HeatEnergy, Length, ThermodynamicTemperature},
    convolution::ElementGridConvolutionNeighborTemperatures,
};

/// The builder of the inputs to the heat propogation system
#[derive(Debug, Clone)]
pub struct PropogateHeatBuilder {
    /// The chunk coordinates
    coords: ChunkCoords,
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
    // /// The compressability of each cell in the chunk
    // compressability: Array2<f32>,
    // /// The total mass above the chunk
    // total_mass_above: Mass,
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
    /// This initializes the propogate heat builder
    pub fn from_element_grid(element_grid: &ElementGrid) -> PropogateHeatBuilder {
        let mut propogate_heat_builder =
            PropogateHeatBuilder::new(*element_grid.get_chunk_coords());
        for j in 0..element_grid.get_chunk_coords().get_num_concentric_circles() {
            for k in 0..element_grid.get_chunk_coords().get_num_radial_lines() {
                let pos = JkVector { j, k };
                let element = element_grid.get_grid().get(pos);

                // Add to the propogate heat builder
                propogate_heat_builder.add(pos, element);
            }
        }
        propogate_heat_builder
    }

    /// Create a new heat propogation system with the given width and height
    /// All the arrays will be initialized to 0
    pub fn new(coords: ChunkCoords) -> Self {
        let width = coords.get_num_radial_lines();
        let height = coords.get_num_concentric_circles();
        let temperature = Array2::from_elem((width, height), 0.0);
        let thermal_conductivity = Array2::from_elem((width, height), 0.0);
        let specific_heat_capacity = Array2::from_elem((width, height), 0.0);
        let density = Array2::from_elem((width, height), 0.0);
        // let compressability = Array2::from_elem((width, height), 0.0);
        Self {
            coords,
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
            delta_multiplier: 1.0,
            enable_compression: false,
            // This will leak a little heat to space over time
            top_temp_mult: 1.0,
            // compressability,
            // total_mass_above: Mass(-1.0),
        }
    }

    /// Add an element to the heat propogation system
    #[allow(clippy::borrowed_box)]
    pub fn add(&mut self, jk_vector: JkVector, elem: &Box<dyn Element>) {
        let density = elem.get_density();
        let specific_heat = elem.get_specific_heat();
        let mass = density.mass(self.coords.get_cell_width());
        let heat_capacity = specific_heat.heat_capacity(mass);
        let idx: [usize; 2] = jk_vector.to_ndarray_coords(&self.coords).into();
        self.temperature[idx] = elem.get_heat().temperature(heat_capacity).0;
        self.thermal_conductivity[idx] = elem.get_thermal_conductivity().0;
        self.specific_heat_capacity[idx] = specific_heat.0;
        self.density[idx] = density.0;
    }

    /// Simple setter for the total mass above the chunk
    // pub fn total_mass_above(&mut self, total_mass_above: Mass) {
    //     self.total_mass_above = total_mass_above;
    // }

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

    /// Get the temperature from a certain cell
    pub fn get_temperature(&self, jk_vector: JkVector) -> ThermodynamicTemperature {
        // Remove the padding
        let temp = self.temperature.slice(s![1..-1, 1..-1]);
        let idx: [usize; 2] = jk_vector.to_ndarray_coords(&self.coords).into();
        ThermodynamicTemperature(temp[idx])
    }

    // /// Set the temperature of the border cells based on the convolved neighbor temperatures
    // /// This is only called by the build method because it needs to be called after all
    // /// adds are done
    // #[allow(clippy::reversed_empty_ranges)]
    // fn border_temperatures(
    //     &mut self,
    //     neighbor_temperatures: ElementGridConvolutionNeighborTemperatures,
    // ) {
    //     // Remember ndarrays are row-major and the LT is 0,0
    //     // so we are going to make some named slices to make this easier
    //     let left_side = s![0, 1..-1];
    //     let right_side = s![-1, 1..-1];
    //     let top_side = s![1..-1, 0];
    //     let second_to_top_side = s![1..-1, 1];
    //     let bottom_side = s![1..-1, -1];
    //     let second_to_bottom_side = s![1..-1, -2];

    //     // Left Right
    //     self.temperature
    //         .slice_mut(left_side)
    //         .assign(&neighbor_temperatures.left.temperature);
    //     self.temperature
    //         .slice_mut(right_side)
    //         .assign(&neighbor_temperatures.right.temperature);
    //     self.density
    //         .slice_mut(left_side)
    //         .assign(&neighbor_temperatures.left.density);
    //     self.density
    //         .slice_mut(right_side)
    //         .assign(&neighbor_temperatures.right.density);
    //     self.specific_heat_capacity
    //         .slice_mut(left_side)
    //         .assign(&neighbor_temperatures.left.specific_heat_capacity);
    //     self.specific_heat_capacity
    //         .slice_mut(right_side)
    //         .assign(&neighbor_temperatures.right.specific_heat_capacity);
    //     self.thermal_conductivity
    //         .slice_mut(left_side)
    //         .assign(&neighbor_temperatures.left.thermal_conductivity);
    //     self.thermal_conductivity
    //         .slice_mut(right_side)
    //         .assign(&neighbor_temperatures.right.thermal_conductivity);

    //     if let Some(top) = neighbor_temperatures.top {
    //         self.temperature.slice_mut(top_side).assign(&top.temperature);
    //         self.density.slice_mut(top_side).assign(&top.density);
    //         self.specific_heat_capacity
    //             .slice_mut(top_side)
    //             .assign(&top.specific_heat_capacity);
    //         self.thermal_conductivity
    //             .slice_mut(top_side)
    //             .assign(&top.thermal_conductivity);
    //     } else {
    //         // Else the top is open to space
    //         // and it will be the same as the next layer down times some multiplier
    //         let second_last_row =
    //             self.temperature.slice(second_to_top_side).to_owned() * self.top_temp_mult;
    //         self.temperature
    //             .slice_mut(top_side)
    //             .assign(&second_last_row);
    //         let second_last_row = self.density.slice(second_to_top_side).to_owned();
    //         self.density.slice_mut(top_side).assign(&second_last_row);
    //         let second_last_row = self.specific_heat_capacity.slice(second_to_top_side).to_owned();
    //         self.specific_heat_capacity.slice_mut(top_side).assign(&second_last_row);
    //         let second_last_row = self.thermal_conductivity.slice(second_to_top_side).to_owned();
    //         self.thermal_conductivity.slice_mut(top_side).assign(&second_last_row);
    //     }
    //     if let Some(bottom) = neighbor_temperatures.bottom {
    //         self.temperature.slice_mut(bottom_side).assign(&bottom.temperature);
    //         self.density.slice_mut(bottom_side).assign(&bottom.density);
    //         self.specific_heat_capacity
    //             .slice_mut(bottom_side)
    //             .assign(&bottom.specific_heat_capacity);
    //         self.thermal_conductivity
    //             .slice_mut(bottom_side)
    //             .assign(&bottom.thermal_conductivity);
    //     } else {
    //         // Else the bottom is the bottom of the world
    //         // so we will set it to the same temp as the next layer up
    //         let second_row = self.temperature.slice(second_to_bottom_side).to_owned();
    //         self.temperature.slice_mut(bottom_side).assign(&second_row);
    //         let second_row = self.density.slice(second_to_bottom_side).to_owned();
    //         self.density.slice_mut(bottom_side).assign(&second_row);
    //         let second_row = self.specific_heat_capacity.slice(second_to_bottom_side).to_owned();
    //         self.specific_heat_capacity.slice_mut(bottom_side).assign(&second_row);
    //         let second_row = self.thermal_conductivity.slice(second_to_bottom_side).to_owned();
    //         self.thermal_conductivity.slice_mut(bottom_side).assign(&second_row);
    //     }

    //     // Now we just need to interpolate the corners for each matrix
    //     Self::interpolate_corners(&mut self.temperature);
    //     Self::interpolate_corners(&mut self.density);
    //     Self::interpolate_corners(&mut self.specific_heat_capacity);
    //     Self::interpolate_corners(&mut self.thermal_conductivity);
    // }

    /// Set the temperature of the border cells based on the convolved neighbor temperatures
    /// This is only called by the build method because it needs to be called after all
    /// adds are done
    fn border_temperatures(
        &mut self,
        neighbor_temperatures: ElementGridConvolutionNeighborTemperatures,
    ) {
        // Define a closure to handle the repeated assignment logic
        let mut assign_property =
            |property: &mut Array2<f32>,
             side: &SliceInfo<[SliceInfoElem; 2], Ix2, Ix1>,
             neighbor_property: &Array1<f32>| {
                property.slice_mut(side).assign(neighbor_property);
            };

        // Slices definitions remain the same
        let left_side = s![0, 1..-1];
        let right_side = s![-1, 1..-1];
        let top_side = s![1..-1, 0];
        let second_to_top_side = s![1..-1, 1];
        let bottom_side = s![1..-1, -1];
        let second_to_bottom_side = s![1..-1, -2];

        // Assign left and right sides for all properties
        for (property, neighbor_side) in [
            (
                &mut self.temperature,
                &neighbor_temperatures.left.temperature,
            ),
            (&mut self.density, &neighbor_temperatures.left.density),
            (
                &mut self.specific_heat_capacity,
                &neighbor_temperatures.left.specific_heat_capacity,
            ),
            (
                &mut self.thermal_conductivity,
                &neighbor_temperatures.left.thermal_conductivity,
            ),
        ]
        .iter_mut()
        {
            assign_property(*property, &left_side, neighbor_side);
        }

        for (property, neighbor_side) in [
            (
                &mut self.temperature,
                &neighbor_temperatures.right.temperature,
            ),
            (&mut self.density, &neighbor_temperatures.right.density),
            (
                &mut self.specific_heat_capacity,
                &neighbor_temperatures.right.specific_heat_capacity,
            ),
            (
                &mut self.thermal_conductivity,
                &neighbor_temperatures.right.thermal_conductivity,
            ),
        ]
        .iter_mut()
        {
            assign_property(*property, &right_side, neighbor_side);
        }

        // Handle top and bottom sides with conditionals
        let sides_with_conditions = [
            (
                neighbor_temperatures.top.as_ref(),
                top_side,
                second_to_top_side,
            ),
            (
                neighbor_temperatures.bottom.as_ref(),
                bottom_side,
                second_to_bottom_side,
            ),
        ];

        for (optional_neighbor, side, second_side) in sides_with_conditions.iter() {
            match optional_neighbor {
                Some(neighbor) => {
                    for (property, neighbor_property) in [
                        (&mut self.temperature, &neighbor.temperature),
                        (&mut self.density, &neighbor.density),
                        (
                            &mut self.specific_heat_capacity,
                            &neighbor.specific_heat_capacity,
                        ),
                        (
                            &mut self.thermal_conductivity,
                            &neighbor.thermal_conductivity,
                        ),
                    ]
                    .iter_mut()
                    {
                        assign_property(*property, side, neighbor_property);
                    }
                }
                None => {
                    for property in [
                        &mut self.temperature,
                        &mut self.density,
                        &mut self.specific_heat_capacity,
                        &mut self.thermal_conductivity,
                    ]
                    .iter_mut()
                    {
                        let second_row = property.slice(*second_side).to_owned();
                        assign_property(*property, side, &second_row);
                    }
                }
            }
        }

        // Interpolate corners for each matrix
        Self::interpolate_corners(&mut self.temperature);
        Self::interpolate_corners(&mut self.density);
        Self::interpolate_corners(&mut self.specific_heat_capacity);
        Self::interpolate_corners(&mut self.thermal_conductivity);
    }

    fn interpolate_corners(matrix: &mut Array2<f32>) {
        // Now we just need to interpolate the corners
        let dim = matrix.dim();
        matrix[[0, 0]] = (matrix[[0, 1]] + matrix[[1, 0]]) / 2.0;
        matrix[[0, dim.1 - 1]] = (matrix[[0, dim.1 - 2]] + matrix[[1, dim.1 - 1]]) / 2.0;
        matrix[[dim.0 - 1, dim.1 - 1]] =
            (matrix[[dim.0 - 1, dim.1 - 2]] + matrix[[dim.0 - 2, dim.1 - 1]]) / 2.0;
        matrix[[dim.0 - 1, 0]] = (matrix[[dim.0 - 1, 1]] + matrix[[dim.0 - 2, 0]]) / 2.0;
    }

    /// Create the structure and test all the values
    pub fn build(
        mut self,
        neighbor_temperatures: ElementGridConvolutionNeighborTemperatures,
    ) -> PropogateHeat {
        // Set the border temperatures
        self.border_temperatures(neighbor_temperatures);
        // Check you called the methods
        // debug_assert!(
        //     self.total_mass_above.0 >= 0.0,
        //     "Total mass above must be greater than or equal to 0. Did you set it?"
        // );
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
            self.thermal_conductivity.dim().0,
            "Temperature must be the size of the thermal conductivity on both hight and width"
        );
        debug_assert_eq!(
            self.temperature.dim().1,
            self.thermal_conductivity.dim().1,
            "Temperature must be the size of the thermal conductivity on both hight and width"
        );
        // debug_assert_eq!(
        //     self.compressability.dim(),
        //     self.thermal_conductivity.dim(),
        //     "Compressability must be the same size as the thermal conductivity"
        // );

        // debug_assert!(
        //     self.total_mass_above.0 >= 0.0,
        //     "Total mass above must be greater than or equal to 0. Did you set it?"
        // );
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
        // debug_assert!(
        //     self.compressability.iter().all(|&x| x.is_finite()),
        //     "Compressability must be finite"
        // );
        debug_assert!(
            self.delta_multiplier.is_finite(),
            "Delta multiplier must be finite"
        );
        debug_assert!(
            self.delta_multiplier > 0.0,
            "Delta multiplier must be greater than 0"
        );
        PropogateHeat {
            cell_width: self.coords.get_cell_width(),
            temperature: self.temperature,
            // total_mass_above: self.total_mass_above,
            thermal_conductivity: self.thermal_conductivity,
            specific_heat_capacity: self.specific_heat_capacity,
            density: self.density,
            // compressability: self.compressability,
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
    // total_mass_above: Mass,
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
    // compressability: Array2<f32>,
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
    /// Constructs the sparse matrix representing the discrete Laplacian operator,
    /// adjusted for the Implicit Euler time-stepping.
    fn construct_laplacian_matrix(&self, current_time: Clock) -> CsMat<f32> {
        let n_rows = self.temperature.dim().0;
        let n_cols = self.temperature.dim().1;
        let n = n_rows * n_cols;
        let mut tri_mat = TriMat::new((n, n));
        let density = {
            if self.enable_compression {
                todo!("Compression is not implemented");
            } else {
                self.density.clone()
            }
        };
        let alpha = &self.thermal_conductivity / (&self.specific_heat_capacity * density);
        // Replace all Nans with zero because anything that has specific heat capacity 0 also has 0 thermal conductivity
        let alpha = alpha.mapv(|x| if x.is_finite() { x } else { 0.0 });
        let coeff = alpha * current_time.get_last_delta().as_secs_f32();

        for i in 0..n_rows {
            for j in 0..n_cols {
                let idx = i * n_cols + j;
                let this_coeff = coeff[[i, j]];
                tri_mat.add_triplet(idx, idx, 1.0 + 4.0 * this_coeff); // Center coefficient

                if i > 0 {
                    tri_mat.add_triplet(idx, idx - n_cols, -this_coeff); // Top
                }
                if i < n_rows - 1 {
                    tri_mat.add_triplet(idx, idx + n_cols, -this_coeff); // Bottom
                }
                if j > 0 {
                    tri_mat.add_triplet(idx, idx - 1, -this_coeff); // Left
                }
                if j < n_cols - 1 {
                    tri_mat.add_triplet(idx, idx + 1, -this_coeff); // Right
                }
            }
        }

        tri_mat.to_csr()
    }

    /// Solves Ax = b using the Conjugate Gradient method.
    /// A: symmetric positive-definite sparse matrix
    /// b: right-hand side vector
    /// tol: tolerance for convergence
    /// Returns a result with the solution x if the solver converges, or an error with the current solution if it does not converge
    pub fn conjugate_gradient(
        A: &CsMat<f32>,
        b: &Array1<f32>,
        tol: f32,
    ) -> Result<Array1<f32>, Array1<f32>> {
        let mut x = Array1::<f32>::zeros(b.len()); // Initial guess x_0 = 0
        let mut r = b.clone(); // Initial residual r_0 = b - Ax_0 = b
        let mut p = r.clone(); // Initial direction p_0 = r_0
        let mut r_norm = r.dot(&r);

        for _ in 0..b.len() {
            // Simple iteration limit
            let ap = A.dot(&p);
            let alpha = r_norm / p.dot(&ap);
            x = x + &(p * alpha);
            let r_new = r - &(ap * alpha);

            let r_new_norm = r_new.dot(&r_new);
            if r_new_norm.sqrt() < tol {
                return Ok(x); // Convergence
            }

            let beta = r_new_norm / r_norm;
            p = r_new + &(p * beta);
            r = r_new;
            r_norm = r_new_norm;
        }

        Err(x) // Did not converge within iteration limit
    }

    /// This is the main method of the heat propogation system
    /// Propogate the heat one iteration
    /// Rerun this method multiple times to propogate the heat multiple iterations
    /// without needing to reinitialize the system
    /// however, movement will not be accounted for if you do this
    fn propagate_heat_implicit_euler(&mut self, current_time: Clock) {
        if current_time.get_last_delta().as_secs_f32() == 0.0 {
            println!("Delta time is 0, not processing heat. May just be the first frame.");
            return;
        }

        let laplacian_matrix = self.construct_laplacian_matrix(current_time);

        // Flatten the current temperature grid to match the dimensions of the Laplacian matrix
        let b = self.temperature.iter().cloned().collect::<Vec<f32>>();
        let b = Array1::from_vec(b);

        // Solve Ax = b for x using the conjugate gradient method
        // Assume a reasonable tolerance for convergence
        let tol = 1e-6;
        let new_temp_flat = Self::conjugate_gradient(&laplacian_matrix, &b, tol);
        let new_temp_flat = match new_temp_flat {
            Ok(x) => x,
            Err(x) => {
                error!("Conjugate gradient did not converge within iteration limit");
                x
            }
        };

        // Reshape the solution back into the 2D grid and update the temperature field
        let (n_rows, n_cols) = self.temperature.dim();
        let new_temp = Array2::from_shape_vec((n_rows, n_cols), new_temp_flat.to_vec())
            .expect("Reshape failed");

        // Update only the interior of the grid to preserve boundary conditions
        // If boundary conditions need to be updated, adjust here accordingly
        self.temperature
            .slice_mut(s![1..-1, 1..-1])
            .assign(&new_temp.slice(s![1..-1, 1..-1]));
    }
    // /// This is the main method of the heat propogation system
    // /// Propogate the heat one iteration
    // /// Rerun this method multiple times to propogate the heat multiple iterations
    // /// without needing to reinitialize the system
    // /// however, movement will not be accounted for if you do this
    // #[allow(clippy::reversed_empty_ranges)] // REF: https://github.com/rust-lang/rust-clippy/issues/5808
    // pub fn propagate_heat(&mut self, current_time: Clock) {
    //     if current_time.get_last_delta().as_secs_f32() == 0.0 {
    //         warn!("Delta time is 0, not processing heat. May just be the first frame.");
    //         return;
    //     }

    //     // Define the convolution kernel
    //     // Apparently it's VERY important that the center be a negative number
    //     let laplace_kernel = Array2::from_shape_vec(
    //         (3, 3),
    //         vec![
    //             1., 1., 1., //
    //             1., -8., 1., //
    //             1., 1., 1., //
    //         ],
    //     )
    //     .unwrap();
    //     debug_assert_eq!(laplace_kernel.sum(), 0.0, "Kernel must sum to 0");

    //     // Convolve the temperature with the kernel to get the gradient
    //     let second_gradient_temperature = self
    //         .temperature
    //         .conv_2d_fft(
    //             &laplace_kernel,
    //             PaddingSize::Valid,
    //             PaddingMode::Zeros, // Doesn't matter in Valid mode
    //         )
    //         .unwrap();
    //     // trace!("Second gradient temperature sum: {}", second_gradient_temperature.sum());

    //     // Check everything is finite
    //     assert!(
    //         second_gradient_temperature.iter().all(|&x| x.is_finite()),
    //         "Second gradient temperature must be finite"
    //     );

    //     // Get the alpha grid
    //     // trace!("Thermal conductivity: {}", self.thermal_conductivity.sum());
    //     // trace!("Specific heat capacity: {}", self.specific_heat_capacity.sum());
    //     let density = {
    //         if self.enable_compression {
    //             todo!("Compression is not implemented");
    //             // Compressability::matrix_get_density_from_mass(
    //             //     &self.compressability,
    //             //     &self.density,
    //             //     self.total_mass_above,
    //             // )
    //         } else {
    //             self.density.clone()
    //         }
    //     };
    //     // trace!("Density: {}", matrix_get_density_from_mass.sum());
    //     let alpha = &self.thermal_conductivity / (&self.specific_heat_capacity * density);
    //     // Replace all Nans with zero because anything that has specific heat capacity 0 also has 0 thermal conductivity
    //     let alpha = alpha.mapv(|x| if x.is_finite() { x } else { 0.0 });
    //     // trace!("Apha sum: {}", alpha.sum());
    //     let delta_temperature = alpha
    //         * second_gradient_temperature
    //         * current_time.get_last_delta().as_secs_f32()
    //         * self.delta_multiplier;

    //     // Check everything is finite
    //     // trace!("Delta temperature sum: {:?}", delta_temperature.sum());
    //     // trace!("time: {:?}", current_time.get_last_delta().as_secs_f32());
    //     assert!(
    //         delta_temperature.iter().all(|&x| x.is_finite()),
    //         "Delta temperature must be finite"
    //     );

    //     // calculate the new temperature
    //     let new_temp = &self.temperature.slice(s![1..-1, 1..-1]) + &delta_temperature;

    //     // Convert to heat
    //     let new_heat_energy = ThermodynamicTemperature::matrix_heat_energy(
    //         &new_temp,
    //         &SpecificHeat::matrix_heat_capacity(
    //             &self.specific_heat_capacity,
    //             &Density::matrix_mass(&self.density, self.cell_width),
    //         ),
    //     );

    //     // Check everything is finite
    //     assert!(
    //         new_heat_energy.iter().all(|&x| x.is_finite()),
    //         "New heat energy must be finite"
    //     );

    //     // Save the new temperature
    //     self.temperature
    //         .slice_mut(s![1..-1, 1..-1])
    //         .assign(&new_temp);
    // }

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
    #[allow(clippy::reversed_empty_ranges)]
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
        let mut builder = PropogateHeatBuilder::new(coords);
        builder.enable_compression(false);
        // builder.total_mass_above(Mass(0.0));
        for j in 0..5 {
            for k in 0..5 {
                builder.add(
                    JkVector::new(j, k),
                    &element_type.get_element(Length(1.0)).box_clone(),
                );
            }
        }

        // This is the border
        let mut heat = builder.build(ElementGridConvolutionNeighborTemperatures {
            left: MatrixBorderHeatProperties {
                temperature: Array1::zeros(5),
                density: Array1::ones(5),
                specific_heat_capacity: Array1::ones(5),
                thermal_conductivity: Array1::ones(5),
            },
            right: MatrixBorderHeatProperties {
                temperature: Array1::zeros(5),
                density: Array1::ones(5),
                specific_heat_capacity: Array1::ones(5),
                thermal_conductivity: Array1::ones(5),
            },
            top: Some(MatrixBorderHeatProperties {
                temperature: Array1::zeros(5),
                density: Array1::ones(5),
                specific_heat_capacity: Array1::ones(5),
                thermal_conductivity: Array1::ones(5),
            }),
            bottom: Some(MatrixBorderHeatProperties {
                temperature: Array1::zeros(5),
                density: Array1::ones(5),
                specific_heat_capacity: Array1::ones(5),
                thermal_conductivity: Array1::ones(5),
            }),
        });

        let mut clock = Clock::default();
        let first_avg = heat.get_temperature().slice(s![1..-1, 1..-1]).sum() / (5 * 5) as f32;
        for frame_cnt in 0..frames {
            let avg = heat.get_temperature().slice(s![1..-1, 1..-1]).sum() / (5 * 5) as f32;
            assert!(
                avg >= (first_avg / 2.0),
                "Took less than {} frames to cool down: {}",
                frames,
                frame_cnt
            );

            // Update the clock
            clock.update(Duration::from_secs_f32(1.0 / frame_rate as f32));

            // Check that the heat is not yet near zero in the center
            // let heat_energy = heat.get_temperature().clone();
            // if frame_cnt % frame_rate == 0 {
            //     println!("#{:?} Heat energy:\n{:?}", frame_cnt, heat_energy);
            // }

            // Propogate the heat
            heat.propagate_heat_implicit_euler(clock);
        }

        // Check that the heat is near zero in the center
        let avg = heat.get_temperature().slice(s![1..-1, 1..-1]).sum() / (5 * 5) as f32;
        assert!(
            avg < (first_avg / 2.0),
            "Took longer than {} frames to cool down.",
            frames
        );
    }
}

#[cfg(test)]
mod tests {
    use super::PropogateHeat;
    use ndarray::Array1;
    use sprs::CsMat;

    #[test]
    fn test_conjugate_gradient() {
        // Example usage
        let a = CsMat::eye(5); // Identity matrix as a simple example
        let b = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let tol = 1e-6;

        if let Ok(solution) = PropogateHeat::conjugate_gradient(&a, &b, tol) {
            println!("Solution: {:?}", solution);
        } else {
            panic!("Solver did not converge");
        }
    }
}
