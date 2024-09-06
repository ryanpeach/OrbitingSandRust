use bevy::color::Color;

use super::element::{Density, Element, ElementTakeOptions, ElementType, StateOfMatter};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {
    last_processed: Clock,
}

impl Element for Vacuum {
    fn element_type(&self) -> ElementType {
        ElementType::Vacuum
    }
    fn last_processed(&self) -> Clock {
        self.last_processed
    }
    fn density(&self) -> Density {
        Density(0.0)
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    fn state_of_matter(&self) -> StateOfMatter {
        StateOfMatter::Empty
    }
    fn color(&self) -> Color {
        Color::srgba(0.0, 0.0, 0.0, 0.0)
    }
    fn _process(
        &mut self,
        _pos: JkVector,
        _coord_dir: &CoordinateDir,
        _target_chunk: &mut ElementGrid,
        _element_grid_conv: &mut ElementGridConvolutionNeighbors,
        _current_time: Clock,
    ) -> ElementTakeOptions {
        ElementTakeOptions::PutBack
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
