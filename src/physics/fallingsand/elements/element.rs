use crate::physics::fallingsand::element_convolution::ElementGridConvolution;
use ggez::graphics::Color;
use uom::si::{f64::Time, time::second};

pub trait Element: Send + Sync {
    fn get_color(&self) -> Color;
    fn process(&mut self, element_grid_conv: &mut ElementGridConvolution, delta: Time);
}
