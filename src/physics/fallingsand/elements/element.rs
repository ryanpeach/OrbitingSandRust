use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_identifiers::ConvolutionIdx;
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::JkVector;
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

/// Useful for match statements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementType {
    Vacuum,
    Sand,
}

pub trait Element: Send + Sync {
    fn get_type(&self) -> ElementType;
    fn get_last_processed(&self) -> Clock;
    #[allow(clippy::borrowed_box)]
    fn get_color(&self, pos: JkVector, chunk_coords: &Box<dyn ChunkCoords>) -> Color;
    fn process(
        &mut self,
        pos: JkVector,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions;
    fn box_clone(&self) -> Box<dyn Element>;

    /// Tries to swap the element with the element at pos1
    /// pos0 should be the position of the element that is being processed
    /// you should have already checked to see if pos1 is valid, most likely it comes from another function
    /// as such this function will panic if pos1 is invalid
    fn try_swap_me(
        &self,
        _pos0: JkVector,
        pos1: ConvolutionIdx,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        let prev = element_grid_conv.replace(target_chunk, pos1, self.box_clone(), current_time);
        match prev {
            Ok(prev) => ElementTakeOptions::ReplaceWith(prev),
            Err(_) => panic!("Tried to swap with an invalid position"),
        }
    }
}
