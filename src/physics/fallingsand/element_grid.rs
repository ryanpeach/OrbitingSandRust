use ggez::graphics::Rect;

use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::elements::element::Element;

use super::elements::vacuum::Vacuum;
use super::util::RawImage;

/// An element grid is a 2D grid of elements tied to a chunk
pub struct ElementGrid {
    grid: Vec<Box<dyn Element>>,
    chunk_coords: Box<dyn ChunkCoords>,
}

impl ElementGrid {
    pub fn new_empty(chunk_coords: Box<dyn ChunkCoords>) -> Self {
        let mut grid: Vec<Box<dyn Element>> = Vec::with_capacity(
            chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles(),
        );
        for _ in 0..chunk_coords.get_num_radial_lines() * chunk_coords.get_num_concentric_circles()
        {
            grid.push(Box::new(Vacuum::default()));
        }
        Self { grid, chunk_coords }
    }

    /// Draw the texture as the color of each element
    pub fn get_texture(&self) -> RawImage {
        let mut out = Vec::with_capacity(
            self.chunk_coords.get_num_radial_lines()
                * self.chunk_coords.get_num_concentric_circles()
                * 4,
        );
        for element in &self.grid {
            let color = element.get_color().to_rgba();
            out.push(color.0);
            out.push(color.1);
            out.push(color.2);
            out.push(color.3);
        }
        RawImage {
            pixels: out,
            bounds: Rect::new(
                self.chunk_coords.get_start_radial_line() as f32,
                self.chunk_coords.get_start_concentric_circle_absolute() as f32,
                self.chunk_coords.get_num_radial_lines() as f32,
                self.chunk_coords.get_num_concentric_circles() as f32,
            ),
        }
    }
}
