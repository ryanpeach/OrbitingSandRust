use ggez::graphics::Color;

use super::element::{Element, ElementTakeOptions};
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_convolution::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::mesh::Square;
use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};
use crate::physics::util::clock::Clock;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {
    last_processed: Clock,
}

impl Element for Vacuum {
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn get_color(&self) -> Color {
        Color::BLACK
    }
    fn get_uv_index(&self) -> u8 {
        0
    }
    fn process(
        &mut self,
        _pos: IjkVector,
        _target_chunk: &mut ElementGrid,
        _element_grid_conv: &mut ElementGridConvolutionNeighbors,
        _current_time: Clock,
    ) -> ElementTakeOptions {
        self.last_processed = _current_time;
        ElementTakeOptions::PutBack
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
