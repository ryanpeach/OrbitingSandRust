use ggez::graphics::Rect;
use uom::si::f64::Time;

use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::elements::element::{Element, ElementTakeOptions};
use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};

use super::coordinates::core_coords::CoreChunkCoords;
use super::element_convolution::ElementGridConvolutionNeighbors;
use super::elements::vacuum::Vacuum;
use super::util::grid::Grid;
use super::util::image::RawImage;

/// An element grid is a 2D grid of elements tied to a chunk
pub struct ElementGrid {
    grid: Grid<Box<dyn Element>>,
    coords: Box<dyn ChunkCoords>,
    already_processed: bool,
}

/// Useful for borrowing the grid to have a default value of one
impl Default for ElementGrid {
    fn default() -> Self {
        Self::new_empty(Box::<CoreChunkCoords>::default())
    }
}

/* Initialization */
impl ElementGrid {
    /// Creates a new element grid with the given chunk coords and fills it with vacuum
    pub fn new_empty(chunk_coords: Box<dyn ChunkCoords>) -> Self {
        let fill: Box<dyn Element> = Box::<Vacuum>::default();
        ElementGrid::new_filled(chunk_coords, &fill)
    }

    /// Creates a new element grid with the given chunk coords and fills it with the given element
    pub fn new_filled(chunk_coords: Box<dyn ChunkCoords>, fill: &Box<dyn Element>) -> Self {
        let mut grid: Vec<Box<dyn Element>> = Vec::with_capacity(
            chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles(),
        );
        for _ in 0..chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles()
        {
            grid.push(fill.box_clone());
        }
        Self {
            grid: Grid::new(
                chunk_coords.get_num_radial_lines(),
                chunk_coords.get_num_concentric_circles(),
                grid,
            ),
            coords: chunk_coords,
            already_processed: false,
        }
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
    #[allow(clippy::borrowed_box)]
    pub fn get_chunk_coords(&self) -> &Box<dyn ChunkCoords> {
        &self.coords
    }
    pub fn get_grid(&self) -> &Grid<Box<dyn Element>> {
        &self.grid
    }
}

/* Handle processing */
impl ElementGrid {
    /// Do one iteration of processing on the grid
    #[allow(clippy::mem_replace_with_default)]
    pub fn process(
        &mut self,
        element_grid_conv_neigh: &mut ElementGridConvolutionNeighbors,
        delta: Time,
    ) {
        let already_processed = self.get_already_processed();
        debug_assert!(!already_processed, "Already processed");
        for j in 0..self.coords.get_num_concentric_circles() {
            for k in 0..self.coords.get_num_radial_lines() {
                let pos = IjkVector {
                    i: self.coords.get_layer_num(),
                    j,
                    k,
                };

                // We have to take the element out of our grid to call it with a reference to self
                // Otherwise we would have a reference to it, and process would have a reference to it through target_chunk
                let mut element = std::mem::replace(
                    self.grid.get_mut(JkVector { j, k }),
                    Box::<Vacuum>::default(),
                );

                // You have to send self and element_grid_conv_neigh my reference instead of packaging them together in an object
                // because you are borrowing both. Without using a lifetime you can't package a borrow.
                let res = element.process(pos, self, element_grid_conv_neigh, delta);

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
                        self.grid.replace(JkVector { j, k }, element);
                    }
                    ElementTakeOptions::ReplaceWith(new_element) => {
                        self.grid.replace(JkVector { j, k }, new_element);
                    }
                    ElementTakeOptions::DoNothing => {}
                }
            }
        }
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
                let color = element
                    .get_color(JkVector { j, k }, self.get_chunk_coords())
                    .to_rgba();
                out.push(color.0);
                out.push(color.1);
                out.push(color.2);
                out.push(color.3);
            }
        }
        RawImage {
            pixels: out,
            bounds: Rect::new(
                self.coords.get_start_radial_line() as f32,
                self.coords.get_start_concentric_circle_absolute() as f32,
                self.coords.get_num_radial_lines() as f32,
                self.coords.get_num_concentric_circles() as f32,
            ),
        }
    }
}
