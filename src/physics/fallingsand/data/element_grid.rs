#![expect(missing_docs)]
#![expect(clippy::missing_docs_in_private_items)]
use bevy::color::ColorToPacked;
use bevy::math::URect;
use conv::ValueFrom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::physics::fallingsand::elements::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::orbits::components::Mass;
use crate::physics::util::clock::Clock;

use super::super::convolution::behaviors::ElementGridConvolutionNeighbors;
use super::super::elements::vacuum::Vacuum;
use super::super::mesh::coordinate_dir::CoordinateDir;
use super::super::util::grid::{GridOutOfBoundsError, JkGrid};
use super::super::util::image::RawImage;
use anyhow::{bail, Result};
use itertools::iproduct;

/// An element grid is a 2D grid of elements tied to a chunk
pub struct ElementGrid {
    grid: JkGrid<Box<dyn Element>>,
    coords: ChunkCoords,

    /// Some low resolution data about the world
    total_mass: Mass, // Total mass in kilograms
    // total_mass_above: Mass, // Total mass above a certain point, in kilograms
    /// The minimum temperature in the chunk that is not 0
    // min_temp: ThermodynamicTemperature,
    /// The maximum temperature in the chunk
    // max_temp: ThermodynamicTemperature,

    /// This deals with a lock during convolution
    already_processed: bool,

    /// This deals with whether or not the element grid needs to be processed
    /// or if it hasn't seen any changes since the last frame maybe you can skip it
    last_set: Clock,
}

/// Useful for borrowing the grid to have a default value of one
impl Default for ElementGrid {
    fn default() -> Self {
        Self::new_empty(ChunkCoords::default())
    }
}

/* Initialization */
impl ElementGrid {
    /// Creates a new element grid with the given chunk coords and fills it with vacuum
    #[must_use]
    pub fn new_empty(chunk_coords: ChunkCoords) -> Self {
        let fill: &dyn Element = &Vacuum::default();
        ElementGrid::new_filled(chunk_coords, fill)
    }

    /// Creates a new element grid with the given chunk coords and fills it with the given element
    pub fn new_filled(chunk_coords: ChunkCoords, fill: &dyn Element) -> Self {
        let mut grid: Vec<Box<dyn Element>> = Vec::with_capacity(
            chunk_coords.num_radial_lines() as usize
                * chunk_coords.num_concentric_circles() as usize,
        );
        for _ in 0..chunk_coords.num_radial_lines() * chunk_coords.num_concentric_circles() {
            grid.push(fill.box_clone());
        }
        Self {
            grid: JkGrid::new_from_vec(
                chunk_coords.num_radial_lines(),
                chunk_coords.num_concentric_circles(),
                grid,
            )
            .expect("We made this grid ourselves"),
            coords: chunk_coords,
            already_processed: false,
            last_set: Clock::default(),
            total_mass: Mass(0.0),
        }
    }
}

/* Getters & Setters */
impl ElementGrid {
    #[must_use]
    pub fn already_processed(&self) -> bool {
        self.already_processed
    }
    pub fn set_already_processed(&mut self, already_processed: bool) {
        self.already_processed = already_processed;
    }
    /// Sets the already processed flag and errors if it is set to the same value twice
    pub fn set_already_processed_deduplicated(&mut self, already_processed: bool) -> Result<()> {
        if self.already_processed == already_processed {
            bail!("Tried to set the same value twice");
        }
        self.already_processed = already_processed;
        Ok(())
    }
    #[must_use]
    pub fn last_set(&self) -> Clock {
        self.last_set
    }
    #[must_use]
    pub fn coords(&self) -> &ChunkCoords {
        &self.coords
    }
    #[must_use]
    pub fn grid(&self) -> &JkGrid<Box<dyn Element>> {
        &self.grid
    }
    /// Does not calculate the total mass, just gets the set value of it
    #[must_use]
    pub fn total_mass(&self) -> Mass {
        self.total_mass
    }

    // /// Recalculate the total mass
    // pub fn recalculate_total_mass(&mut self) {
    //     let mut mass = Mass(0.0);
    //     for j in 0..self.coords.get_num_concentric_circles() {
    //         for k in 0..self.coords.get_num_radial_lines() {
    //             let pos = JkVector { j, k };
    //             mass += self.grid.get(pos).get_mass(self.coords.get_cell_width());
    //         }
    //     }
    //     self.total_mass = mass;
    // }

    /// Get the maximum temperature in the directory
    // pub fn get_max_min_temp(&self) -> (ThermodynamicTemperature, ThermodynamicTemperature) {
    //     (self.max_temp, self.min_temp)
    // }

    // /// Recalculate the maximum temperature in the directory
    // pub fn recalculate_max_min_temp(&mut self) {
    //     (self.max_temp, self.min_temp) = self.calc_max_min_temp();
    // }

    /// Calculate the maximum temperature in the directory
    // pub fn calc_max_min_temp(&self) -> (ThermodynamicTemperature, ThermodynamicTemperature) {
    //     let mut max = ThermodynamicTemperature(0.0);
    //     let mut min = ThermodynamicTemperature(f32::INFINITY);
    //     for k in 0..self.coords.get_num_radial_lines() {
    //         for j in 0..self.coords.get_num_concentric_circles() {
    //             let pos = JkVector { j, k };
    //             let element = self.grid.get(pos);
    //             let temp = element.get_temperature(self.coords.get_cell_width());
    //             if temp > max {
    //                 max = temp;
    //             }
    //             // If the temperature is 0, we don't want to include it in the minimum
    //             // Because its usually a vaccuum
    //             if temp < min && temp != ThermodynamicTemperature(0.0) {
    //                 min = temp;
    //             }
    //         }
    //     }
    //     (max, min)
    // }

    /// Does not calculate the total mass, just gets the set value of it
    // pub fn get_total_mass_above(&self) -> Mass {
    //     self.total_mass_above
    // }

    #[must_use]
    pub fn process_unneeded(&self, current_time: Clock) -> bool {
        self.last_set.current_frame() < current_time.current_frame() - 1
    }
}

/// Public modifiers for the element grid
impl ElementGrid {
    #[must_use]
    pub fn get(&self, jk: JkVector) -> &dyn Element {
        self.grid.get(jk).as_ref()
    }
    pub fn checked_get(&self, jk: JkVector) -> Result<&dyn Element, GridOutOfBoundsError> {
        match self.grid.checked_get(jk) {
            Ok(e) => Ok(e.as_ref()),
            Err(e) => Err(e),
        }
    }
    pub fn get_mut(&mut self, jk: JkVector) -> &mut dyn Element {
        self.grid.get_mut(jk).as_mut()
    }
    pub fn set(&mut self, jk: JkVector, element: Box<dyn Element>, time: Clock) {
        self.replace(jk, element, time);
    }
    pub fn replace(
        &mut self,
        jk: JkVector,
        element: Box<dyn Element>,
        time: Clock,
    ) -> Box<dyn Element> {
        self.last_set = time;
        self.grid.replace(jk, element)
    }
}

/// Proceedural generation helpers
impl ElementGrid {
    /// Fill the grid with the given element
    pub fn fill(&mut self, element: ElementType) {
        for j in 0..self.coords().num_concentric_circles() {
            for k in 0..self.coords().num_radial_lines() {
                let pos = JkVector { j, k };
                self.grid.replace(pos, element.element());
            }
        }
    }
}

/// Handle processing
impl ElementGrid {
    /// Do one iteration of processing on the grid
    pub fn process(
        &mut self,
        coord_dir: &CoordinateDir,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) {
        self.process_elements(coord_dir, element_grid_conv_neigh, current_time);
        // self.process_heat(element_grid_conv_neigh, current_time);
        self.process_mass(element_grid_conv_neigh);
    }

    /// Run each elements process method
    fn process_elements(
        &mut self,
        coord_dir: &CoordinateDir,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) {
        let already_processed = self.already_processed();
        debug_assert!(!already_processed, "Already processed");

        // By randomly shuffling the order we process the elements
        // we can avoid creating a "favorite direction" for the elements to move
        let mut rng = thread_rng();
        let mut iter: Vec<(u32, u32)> = iproduct!(
            0..self.coords.num_concentric_circles(),
            0..self.coords.num_radial_lines()
        )
        .collect();
        iter.shuffle(&mut rng);
        for (j, k) in iter {
            let pos = JkVector { j, k };

            // We have to take the element out of our grid to call it with a reference to self
            // Otherwise we would have a reference to it, and process would have a reference to it through target_chunk
            let mut element = self.grid.replace(pos, Box::<Vacuum>::default());

            // Check that the element hasn't already been processed this frame
            if element.last_processed().current_frame() >= current_time.current_frame() {
                self.grid.replace(pos, element);
                continue;
            }

            // You have to send self and element_grid_conv_neigh my reference instead of packaging them together in an object
            // because you are borrowing both. Without using a lifetime you can't package a borrow.
            let res = element.process(pos, coord_dir, self, element_grid_conv_neigh, current_time);

            // The reason we return options instead of passing the element to process by value (letting it put itself back) is twofold
            // The first is this prevents the common programming error where the author forgets that the element
            // has been moved out of the grid, and it disappears.
            // The second is that you get a "cant borrow twice" error if you pass the element to process by value
            // It was really complicated to get this to work, so I'm not going to change it.
            // If you try to change it, increment this counter by how may hours you spent trying to change it
            //
            // +1h wasted
            //
            match res {
                ElementTakeOptions::PutBack => {
                    self.grid.replace(pos, element);
                }
                ElementTakeOptions::ReplaceWith(new_element) => {
                    self.grid.replace(pos, new_element);
                }
                ElementTakeOptions::DoNothing => {}
            }
        }
    }

    /// Process the heat of the grid
    /// Currently disabled as it is broken
    // fn process_heat(
    //     &mut self,
    //     element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
    //     current_time: Clock,
    // ) {
    //     let mut propogate_heat_builder = PropogateHeatBuilder::new(self.coords);
    //     let avg_neigh_temp = element_grid_conv_neigh.get_border_temps(self);
    //     for j in 0..self.coords.get_num_concentric_circles() {
    //         for k in 0..self.coords.get_num_radial_lines() {
    //             let pos = JkVector { j, k };
    //             let element = self.grid.get(pos);

    //             // Add to the propogate heat builder
    //             propogate_heat_builder.add(self.get_chunk_coords(), pos, element);
    //         }
    //     }
    //     // Set the total mass above
    //     // propogate_heat_builder.total_mass_above(self.total_mass_above);

    //     // Now build and propogate updates to the element grid
    //     let mut propogate_heat = propogate_heat_builder.build(avg_neigh_temp);
    //     propogate_heat.propagate_heat(current_time);
    //     propogate_heat.apply_to_grid(self, element_grid_conv_neigh, current_time);
    //     // (self.max_temp, self.min_temp) = self.calc_max_min_temp();
    // }

    /// Process the mass of the grid and the mass above the grid
    fn process_mass(&mut self, _element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors) {
        // self.total_mass_above = {
        //     match &element_grid_conv_neigh.grids.top {
        //         TopNeighborGrids::Normal { t, .. } => t.get_total_mass_above() + t.get_total_mass(),
        //         TopNeighborGrids::ChunkDoubling { tl, tr, .. } => {
        //             tl.get_total_mass_above()
        //                 + tl.get_total_mass()
        //                 + tr.get_total_mass_above()
        //                 + tr.get_total_mass()
        //         }
        //         TopNeighborGrids::TopOfGrid => Mass(0.0),
        //     }
        // };
        self.total_mass = (0..self.coords.num_concentric_circles())
            .into_par_iter()
            .map(|j| -> Mass {
                (0..self.coords.num_radial_lines())
                    .into_par_iter()
                    .map(|k| {
                        let pos = JkVector { j, k };
                        let element = self.grid.get(pos);

                        element.mass(self.coords.cell_width())
                    })
                    .sum()
            })
            .sum();
    }

    // Get the heat properties of an element at an index
    // pub fn get_heat_properties(&self, idx: JkVector) -> ElementHeatProperties {
    //     let element = self.grid.get(idx);
    //     element.get_heat_properties(self.coords.get_cell_width())
    // }
}

/* Drawing */
impl ElementGrid {
    /// Draw the texture as the color of each element
    #[must_use]
    pub fn texture(&self) -> RawImage {
        let mut out = Vec::with_capacity(
            self.coords.num_radial_lines() as usize
                * self.coords.num_concentric_circles() as usize
                * 4,
        );
        for j in 0..self.coords.num_concentric_circles() {
            for k in 0..self.coords.num_radial_lines() {
                let element = self.grid.get(JkVector { j, k });
                let color = element.color().to_srgba().to_u8_array();
                out.push(color[0]);
                out.push(color[1]);
                out.push(color[2]);
                out.push(color[3]);
            }
        }
        RawImage {
            pixels: out,
            bounds: URect::new(
                u32::value_from(self.coords.start_radial_line()).expect("Very large number"),
                u32::value_from(self.coords.start_concentric_circle_absolute())
                    .expect("Very large number"),
                u32::value_from(self.coords.start_radial_line() + self.coords.num_radial_lines())
                    .expect("Very large number"),
                u32::value_from(
                    self.coords.start_concentric_circle_absolute()
                        + self.coords.num_concentric_circles(),
                )
                .expect("Very large number."),
            ),
        }
    }

    // Get the texture of the grid as to its heat
    // max_temp is the maximum temperature of the entire directory
    // min_temp is the minimum temperature of the entire directory
    // pub fn get_heat_texture(
    //     &self,
    //     // max_temp: ThermodynamicTemperature,
    //     // min_temp: ThermodynamicTemperature,
    // ) -> RawImage {
    //     let mut out = Vec::with_capacity(
    //         self.coords.get_num_radial_lines() * self.coords.get_num_concentric_circles() * 4,
    //     );
    //     let cell_width = self.coords.get_cell_width();
    //     for j in 0..self.coords.get_num_concentric_circles() {
    //         for k in 0..self.coords.get_num_radial_lines() {
    //             let element = self.grid.get(JkVector { j, k });
    //             let color = element.get_temperature(cell_width).color().as_rgba_u8();
    //             out.push(color[0]);
    //             out.push(color[1]);
    //             out.push(color[2]);
    //             out.push(color[3]);
    //         }
    //     }
    //     RawImage {
    //         pixels: out,
    //         bounds: Rect::new(
    //             self.coords.get_start_radial_line() as f32,
    //             self.coords.get_start_concentric_circle_absolute() as f32,
    //             self.coords.get_start_radial_line() as f32
    //                 + self.coords.get_num_radial_lines() as f32,
    //             self.coords.get_start_concentric_circle_absolute() as f32
    //                 + self.coords.get_num_concentric_circles() as f32,
    //         ),
    //     }
    // }

    // /// Save the grid
    // /// dir_path is the path to the directory where the grid will be saved WITHOUT a trailing slash
    // pub fn save(&self, ctx: &mut ggez::Context, dir_path: &str) -> Result<(), ggez::GameError> {
    //     let idx = self.get_chunk_coords().get_chunk_idx();
    //     let chunk_path = format!("{}/i{}_j{}_k{}.png", dir_path, idx.i, idx.j, idx.k);
    //     self.get_texture().save(ctx, chunk_path.as_str())
    // }
}
