use std::time::Duration;

use super::element::{Element, ElementTakeOptions};
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_convolution::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};
use ggez::graphics::Color;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Sand {
    last_processed: Duration,
}

impl Element for Sand {
    fn get_last_processed(&self) -> Duration {
        self.last_processed
    }
    #[allow(clippy::borrowed_box)]
    fn get_color(&self, _pos: JkVector, _chunk_coords: &Box<dyn ChunkCoords>) -> Color {
        Color::YELLOW
    }
    fn process(
        &mut self,
        _pos: IjkVector,
        _target_chunk: &mut ElementGrid,
        _element_grid_conv: &mut ElementGridConvolutionNeighbors,
        _current_time: Duration,
    ) -> ElementTakeOptions {
        self.last_processed = _current_time;
        ElementTakeOptions::PutBack
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
