use crate::physics::fallingsand::element_convolution::ElementGridConvolution;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::IjkVector;
use ggez::graphics::Color;
use uom::si::f64::Time;

/// What to do after process is called on the elementgrid
/// The element grid takes the element out of the grid so that it can't
/// self reference in the process operation for thread safety.
/// However, we shouldn't put it back if the element has moved, instead
/// we will ask the element itself to clone itself and put the clone somewhere else
pub enum ElementTakeOptions {
    PutBack,
    DoNothing,
    ReplaceWith(Box<dyn Element>),
}

pub trait Element: Send + Sync {
    fn get_color(&self) -> Color;
    fn process(
        &mut self,
        pos: IjkVector,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolution,
        delta: Time,
    ) -> ElementTakeOptions;
}
