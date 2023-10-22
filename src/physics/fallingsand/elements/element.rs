use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;

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
    fn get_last_processed(&self) -> Clock;
    #[allow(clippy::borrowed_box)]
    fn get_color(&self, pos: JkVector, chunk_coords: &Box<dyn ChunkCoords>) -> Color;
    fn process(
        &mut self,
        pos: IjkVector,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions;
    fn box_clone(&self) -> Box<dyn Element>;
}
