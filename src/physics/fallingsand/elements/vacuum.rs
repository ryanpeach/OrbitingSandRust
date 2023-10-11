use ggez::graphics::Color;
use uom::si::{f64::Time, time::second};

use crate::physics::fallingsand::element_convolution::ElementGridConvolution;

use super::element::Element;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {}

impl Element for Vacuum {
    fn get_color(&self) -> Color {
        Color::new(1.0, 0.0, 0.0, 1.0)
    }
    fn process(&mut self, _element_grid_conv: &mut ElementGridConvolution, _delta: Time) {}
}
