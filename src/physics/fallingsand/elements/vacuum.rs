use ggez::graphics::Color;
use uom::si::f64::Time;

use super::element::{Element, ElementTakeOptions};
use crate::physics::fallingsand::element_convolution::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::IjkVector;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {}

impl Element for Vacuum {
    fn get_color(&self) -> Color {
        Color::new(1.0, 0.0, 0.0, 1.0)
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
