use ggez::graphics::Rect;
use uom::si::f64::Time;
use uom::si::time::second;

use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::elements::element::Element;

use super::element_convolution::ElementGridConvolution;
use super::elements::vacuum::Vacuum;
use super::util::grid::Grid;
use super::util::image::RawImage;

/// An element grid is a 2D grid of elements tied to a chunk
pub struct ElementGrid {
    grid: Grid<Box<dyn Element>>,
    coords: Box<dyn ChunkCoords>,
    already_processed: bool,
}

/* Initialization */
impl ElementGrid {
    pub fn new_empty(chunk_coords: Box<dyn ChunkCoords>) -> Self {
        let mut grid: Vec<Box<dyn Element>> = Vec::with_capacity(
            chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles(),
        );
        for _ in 0..chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles()
        {
            grid.push(Box::<Vacuum>::default());
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
    pub fn process(&mut self, element_grid_conv: &mut ElementGridConvolution, delta: Time) {
        for element in self.grid.iter_mut() {
            debug_assert!(!self.get_already_processed(), "Already processed");
            element.process(element_grid_conv, delta);
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
        for element in self.grid.get_data() {
            let color = element.get_color().to_rgba();
            out.push(color.0);
            out.push(color.1);
            out.push(color.2);
            out.push(color.3);
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
