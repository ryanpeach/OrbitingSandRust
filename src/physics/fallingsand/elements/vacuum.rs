use ggez::graphics::Color;
use uom::si::f64::Time;

use super::element::{Element, ElementTakeOptions};
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_convolution::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {}

impl Element for Vacuum {
    #[allow(clippy::borrowed_box)]
    fn get_color(&self, _pos: JkVector, chunk_coords: &Box<dyn ChunkCoords>) -> Color {
        let x = chunk_coords.get_chunk_idx().i
            + chunk_coords.get_chunk_idx().j
            + chunk_coords.get_chunk_idx().k;
        match x % 2 == 0 {
            true => Color::new(0.0, 0.0, 0.0, 1.0),
            false => Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
    fn process(
        &mut self,
        _pos: IjkVector,
        _target_chunk: &mut ElementGrid,
        _element_grid_conv: &mut ElementGridConvolutionNeighbors,
        _delta: Time,
    ) -> ElementTakeOptions {
        ElementTakeOptions::PutBack
    }
}