
use bevy::math::Rect;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::physics::fallingsand::convolution::neighbor_grids::TopNeighborGrids;
use crate::physics::fallingsand::elements::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{
    HeatCapacity, HeatEnergy, ThermodynamicTemperature,
};
use crate::physics::heat::math::PropogateHeatBuilder;
use crate::physics::orbits::components::Mass;
use crate::physics::util::clock::Clock;

use super::super::convolution::behaviors::ElementGridConvolutionNeighbors;

use super::super::elements::vacuum::Vacuum;
use super::super::mesh::coordinate_directory::CoordinateDir;
use super::super::util::grid::{Grid, GridOutOfBoundsError};
use super::super::util::image::RawImage;

/// An element grid is a 2D grid of elements tied to a chunk
pub struct ElementGrid {
    grid: Grid<Box<dyn Element>>,
    coords: ChunkCoords,

    /// Some low resolution data about the world
    total_mass: Mass, // Total mass in kilograms
    total_mass_above: Mass, // Total mass above a certain point, in kilograms
    total_heat: HeatEnergy, // Total heat in joules
    total_heat_capacity_at_atp: HeatCapacity, // Total heat capacity at ATP in joules per kelvin

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
    pub fn new_empty(chunk_coords: ChunkCoords) -> Self {
        let fill: &dyn Element = &Vacuum::default();
        ElementGrid::new_filled(chunk_coords, fill)
    }

    /// Creates a new element grid with the given chunk coords and fills it with the given element
    pub fn new_filled(chunk_coords: ChunkCoords, fill: &dyn Element) -> Self {
        let mut grid: Vec<Box<dyn Element>> = Vec::with_capacity(
            chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles(),
        );
        for _ in 0..chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles()
        {
            grid.push(fill.box_clone());
        }
        let mut out = Self {
            grid: Grid::new_from_vec(
                chunk_coords.get_num_radial_lines(),
                chunk_coords.get_num_concentric_circles(),
                grid,
            ),
            coords: chunk_coords,
            already_processed: false,
            last_set: Clock::default(),
            total_mass: Mass(0.0),
            total_heat_capacity_at_atp: HeatCapacity(0.0),
            total_heat: HeatEnergy(0.0),
            total_mass_above: Mass(0.0),
        };
        out.recalculate_everything();
        out
    }
}

/* Getters & Setters */
impl ElementGrid {
    pub fn get_already_processed(&self) -> bool {
        self.already_processed
    }
    pub fn set_already_processed(&mut self, already_processed: bool) {
        self.already_processed = already_processed;
    }
    /// Sets the already processed flag and errors if it is set to the same value twice
    pub fn set_already_processed_deduplicated(
        &mut self,
        already_processed: bool,
    ) -> Result<(), String> {
        if self.already_processed == already_processed {
            return Err("Tried to set the same value twice".to_string());
        }
        self.already_processed = already_processed;
        Ok(())
    }
    pub fn get_last_set(&self) -> Clock {
        self.last_set
    }
    pub fn get_chunk_coords(&self) -> &ChunkCoords {
        &self.coords
    }
    pub fn get_grid(&self) -> &Grid<Box<dyn Element>> {
        &self.grid
    }
    /// Does not calculate the total mass, just gets the set value of it
    pub fn get_total_mass(&self) -> Mass {
        self.total_mass
    }

    /// Recalculate the total mass
    pub fn recalculate_total_mass(&mut self) {
        let mut mass = Mass(0.0);
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let pos = JkVector { j, k };
                mass += self.grid.get(pos).get_mass(self.coords.get_cell_width());
            }
        }
        self.total_mass = mass;
    }

    /// Recalculate the heat values
    pub fn recalculate_heat(&mut self) {
        let mut heat = HeatEnergy(0.0);
        let mut heat_capacity = HeatCapacity(0.0);
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let pos = JkVector { j, k };
                let element = self.grid.get(pos);
                let mass = element.get_mass(self.coords.get_cell_width());
                let this_heat_capacity = element.get_specific_heat().heat_capacity(mass);
                let this_heat = element.get_heat();

                // Add to the top level variables
                heat += this_heat;
                heat_capacity += this_heat_capacity;
            }
        }
        self.total_heat = heat;
        self.total_heat_capacity_at_atp = heat_capacity;
    }

    /// Recalculate everything without processing
    pub fn recalculate_everything(&mut self) {
        self.recalculate_total_mass();
        self.recalculate_heat();
    }

    /// Does not calculate the total mass, just gets the set value of it
    pub fn get_total_mass_above(&self) -> Mass {
        self.total_mass_above
    }

    /// Get the total heat capacity at ATP
    pub fn get_total_heat_capacity(&self) -> HeatCapacity {
        self.total_heat_capacity_at_atp
    }

    /// Get the total heat
    pub fn get_total_heat(&self) -> HeatEnergy {
        self.total_heat
    }

    /// Get temperature
    /// Assume total_mass_above correlates to pressure
    pub fn get_temperature(&self) -> ThermodynamicTemperature {
        if self.get_total_heat_capacity().0 == 0.0 {
            return ThermodynamicTemperature(0.0);
        }
        ThermodynamicTemperature(self.get_total_heat().0 / self.get_total_heat_capacity().0)
    }
    pub fn get_process_unneeded(&self, current_time: Clock) -> bool {
        self.last_set.get_current_frame() < current_time.get_current_frame() - 1
    }
}

/// Public modifiers for the element grid
impl ElementGrid {
    #[allow(clippy::borrowed_box)]
    pub fn get(&self, jk: JkVector) -> &Box<dyn Element> {
        self.grid.get(jk)
    }
    #[allow(clippy::borrowed_box)]
    pub fn checked_get(&self, jk: JkVector) -> Result<&Box<dyn Element>, GridOutOfBoundsError> {
        self.grid.checked_get(jk)
    }
    #[allow(clippy::borrowed_box)]
    pub fn get_mut(&mut self, jk: JkVector) -> &mut Box<dyn Element> {
        self.grid.get_mut(jk)
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
        for j in 0..self.get_chunk_coords().get_num_concentric_circles() {
            for k in 0..self.get_chunk_coords().get_num_radial_lines() {
                let pos = JkVector { j, k };
                self.grid
                    .replace(pos, element.get_element(self.coords.get_cell_width()));
            }
        }
    }
}

/// Handle processing
impl ElementGrid {
    /// Do one iteration of processing on the grid
    #[allow(clippy::mem_replace_with_default)]
    pub fn process(
        &mut self,
        coord_dir: &CoordinateDir,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) {
        self.process_elements(coord_dir, element_grid_conv_neigh, current_time);
        self.process_heat(element_grid_conv_neigh, current_time);
        self.process_mass(element_grid_conv_neigh);
    }

    /// Run each elements process method
    #[allow(clippy::mem_replace_with_default)]
    fn process_elements(
        &mut self,
        coord_dir: &CoordinateDir,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) {
        // Calculate the mass of the grid as we go
        let already_processed = self.get_already_processed();
        debug_assert!(!already_processed, "Already processed");
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let pos = JkVector { j, k };

                // We have to take the element out of our grid to call it with a reference to self
                // Otherwise we would have a reference to it, and process would have a reference to it through target_chunk
                let mut element = self.grid.replace(pos, Box::<Vacuum>::default());

                // Check that the element hasn't already been processed this frame
                if element.get_last_processed().get_current_frame()
                    >= current_time.get_current_frame()
                {
                    self.grid.replace(pos, element);
                    continue;
                }

                // You have to send self and element_grid_conv_neigh my reference instead of packaging them together in an object
                // because you are borrowing both. Without using a lifetime you can't package a borrow.
                let res =
                    element.process(pos, coord_dir, self, element_grid_conv_neigh, current_time);

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
    }

    /// Process the heat of the grid
    fn process_heat(
        &mut self,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) {
        self.total_heat = HeatEnergy(0.0);
        self.total_heat_capacity_at_atp = HeatCapacity(0.0);
        let mut propogate_heat_builder = PropogateHeatBuilder::new(
            self.coords.get_num_concentric_circles(),
            self.coords.get_num_radial_lines(),
            self.coords.get_cell_width(),
        );
        let avg_neigh_temp = element_grid_conv_neigh.get_avg_temp();
        propogate_heat_builder.border_temperatures(avg_neigh_temp);
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let pos = JkVector { j, k };
                let element = self.grid.get(pos);
                let mass = element.get_mass(self.coords.get_cell_width());
                let heat_capacity = element.get_specific_heat().heat_capacity(mass);
                let heat = element.get_heat();

                // Add to the top level variables
                self.total_heat += heat;
                self.total_heat_capacity_at_atp += heat_capacity;

                // Add to the propogate heat builder
                propogate_heat_builder.add(pos, element);
            }
        }
        // Set the total mass above
        propogate_heat_builder.total_mass_above(self.total_mass_above);

        // Now build and propogate updates to the element grid
        let propogate_heat = propogate_heat_builder.build();
        propogate_heat.propagate_heat(self, current_time);
    }

    /// Process the mass of the grid and the mass above the grid
    fn process_mass(&mut self, element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors) {
        self.total_mass_above = {
            match &element_grid_conv_neigh.grids.top {
                TopNeighborGrids::Normal { t, .. } => t.get_total_mass_above() + t.get_total_mass(),
                TopNeighborGrids::LayerTransition { tl, tr, .. } => {
                    tl.get_total_mass_above()
                        + tl.get_total_mass()
                        + tr.get_total_mass_above()
                        + tr.get_total_mass()
                }
                TopNeighborGrids::TopOfGrid => Mass(0.0),
            }
        };
        self.total_mass = (0..self.coords.get_num_concentric_circles())
            .into_par_iter()
            .map(|j| -> Mass {
                (0..self.coords.get_num_radial_lines())
                    .into_par_iter()
                    .map(|k| {
                        let pos = JkVector { j, k };
                        let element = self.grid.get(pos);
                        
                        element.get_mass(self.coords.get_cell_width())
                    })
                    .sum()
            })
            .sum()
    }
}

/* Drawing */
impl ElementGrid {
    /// Draw the texture as the color of each element
    pub fn get_texture(&self) -> RawImage {
        let mut out = Vec::with_capacity(
            self.coords.get_num_radial_lines() * self.coords.get_num_concentric_circles() * 4,
        );
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let element = self.grid.get(JkVector { j, k });
                let color = element.get_color().as_rgba_u8();
                out.push(color[0]);
                out.push(color[1]);
                out.push(color[2]);
                out.push(color[3]);
            }
        }
        RawImage {
            pixels: out,
            bounds: Rect::new(
                self.coords.get_start_radial_line() as f32,
                self.coords.get_start_concentric_circle_absolute() as f32,
                self.coords.get_start_radial_line() as f32
                    + self.coords.get_num_radial_lines() as f32,
                self.coords.get_start_concentric_circle_absolute() as f32
                    + self.coords.get_num_concentric_circles() as f32,
            ),
        }
    }

    /// Get the texture of the grid as to its heat
    /// max_temp is the maximum temperature of the entire directory
    /// min_temp is the minimum temperature of the entire directory
    pub fn get_heat_texture(
        &self,
        max_temp: ThermodynamicTemperature,
        min_temp: ThermodynamicTemperature,
    ) -> RawImage {
        let mut out = Vec::with_capacity(
            self.coords.get_num_radial_lines() * self.coords.get_num_concentric_circles() * 4,
        );
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let element = self.grid.get(JkVector { j, k });
                let mass = element.get_mass(self.coords.get_cell_width());
                let color = element
                    .get_heat()
                    .temperature(element.get_specific_heat().heat_capacity(mass))
                    .color(max_temp, min_temp)
                    .as_rgba_u8();
                out.push(color[0]);
                out.push(color[1]);
                out.push(color[2]);
                out.push(color[3]);
            }
        }
        RawImage {
            pixels: out,
            bounds: Rect::new(
                self.coords.get_start_radial_line() as f32,
                self.coords.get_start_concentric_circle_absolute() as f32,
                self.coords.get_start_radial_line() as f32
                    + self.coords.get_num_radial_lines() as f32,
                self.coords.get_start_concentric_circle_absolute() as f32
                    + self.coords.get_num_concentric_circles() as f32,
            ),
        }
    }

    // /// Save the grid
    // /// dir_path is the path to the directory where the grid will be saved WITHOUT a trailing slash
    // pub fn save(&self, ctx: &mut ggez::Context, dir_path: &str) -> Result<(), ggez::GameError> {
    //     let idx = self.get_chunk_coords().get_chunk_idx();
    //     let chunk_path = format!("{}/i{}_j{}_k{}.png", dir_path, idx.i, idx.j, idx.k);
    //     self.get_texture().save(ctx, chunk_path.as_str())
    // }
}
