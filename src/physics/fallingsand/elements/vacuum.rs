use ggez::graphics::Color;

use super::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {
    last_processed: Clock,
}

impl Element for Vacuum {
    fn get_type(&self) -> ElementType {
        ElementType::Vacuum
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    #[allow(clippy::borrowed_box)]
    fn get_color(&self) -> Color {
        Color::BLACK
    }
    fn _process(
        &mut self,
        _pos: JkVector,
        _coord_dir: &CoordinateDir,
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
