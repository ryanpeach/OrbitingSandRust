use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_identifiers::ConvolutionIdx;
use crate::physics::fallingsand::coordinates::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;
use strum_macros::EnumIter;

use super::fliers::down::DownFlier;
use super::sand::Sand;
use super::vacuum::Vacuum;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ElementType {
    Vacuum,
    Sand,
    DownFlier,
}

impl ElementType {
    pub fn get_element(&self) -> Box<dyn Element> {
        match self {
            ElementType::Vacuum => Box::<Vacuum>::default(),
            ElementType::Sand => Box::<Sand>::default(),
            ElementType::DownFlier => Box::<DownFlier>::default(),
        }
    }
}

pub trait Element: Send + Sync {
    fn get_type(&self) -> ElementType;
    fn get_last_processed(&self) -> Clock;
    #[allow(clippy::borrowed_box)]
    fn get_color(&self) -> Color;
    fn process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
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

#[cfg(test)]
mod tests {
    use ggegui::egui::epaint::ahash::{HashSet, HashSetExt};
    use ggez::graphics::Color;
    use strum::IntoEnumIterator;

    use super::ElementType;

    #[test]
    fn test_all_elements_have_different_color() {
        let mut colors = Vec::<Color>::new();
        for element_type in ElementType::iter() {
            let color = element_type.get_element().get_color();
            assert!(
                !colors.contains(&color),
                "Color {:?} of element {:?} is not unique",
                color,
                element_type
            );
            colors.push(color);
        }
    }
}
