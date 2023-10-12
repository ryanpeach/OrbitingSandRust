use crate::physics::fallingsand::element_convolution::ElementGridConvolution;
use crate::physics::fallingsand::element_grid::ElementGrid;
use ggez::graphics::Color;
use uom::si::f64::Time;

pub trait Element: Send + Sync {
    fn get_color(&self) -> Color;
    fn process(
        &mut self,
        element_grid: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolution,
        delta: Time,
    );
}
