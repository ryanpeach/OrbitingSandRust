#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_identifiers::ConvolutionIdx;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{HeatCapacity, HeatEnergy};
use crate::physics::orbits::components::Mass;
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;
use strum_macros::EnumIter;

use super::fliers::down::DownFlier;
use super::fliers::left::LeftFlier;
use super::fliers::right::RightFlier;
use super::sand::Sand;
use super::solarplasma::SolarPlasma;
use super::stone::Stone;
use super::vacuum::Vacuum;
use super::water::Water;

#[derive(Default)]
pub struct Density(pub f32);

/// What to do after process is called on the elementgrid
/// The element grid takes the element out of the grid so that it can't
/// self reference in the process operation for thread safety.
/// However, we shouldn't put it back if the element has moved, instead
/// we will ask the element itself to clone itself and put the clone somewhere else
#[derive(Default)]
pub enum ElementTakeOptions {
    #[default]
    PutBack,
    DoNothing,
    ReplaceWith(Box<dyn Element>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, PartialOrd, Ord)]
pub enum StateOfMatter {
    #[default]
    Empty,
    Gas,
    Liquid,
    Solid,
}

/// Useful for match statements
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ElementType {
    #[default]
    Vacuum,
    Sand,
    Stone,
    Water,
    SolarPlasma,
    DownFlier,
    LeftFlier,
    RightFlier,
}

impl ElementType {
    pub fn get_element(&self) -> Box<dyn Element> {
        match self {
            ElementType::Vacuum => Box::<Vacuum>::default(),
            ElementType::Sand => Box::<Sand>::default(),
            ElementType::DownFlier => Box::<DownFlier>::default(),
            ElementType::LeftFlier => Box::<LeftFlier>::default(),
            ElementType::RightFlier => Box::<RightFlier>::default(),
            ElementType::Stone => Box::<Stone>::default(),
            ElementType::SolarPlasma => Box::<SolarPlasma>::default(),
            ElementType::Water => Box::<Water>::default(),
        }
    }
}

/// If something has 0 heat capacity, you should not set its heat
pub struct SetHeatOnZeroHeatCapacityError;

pub trait Element: Send + Sync {
    fn get_type(&self) -> ElementType;
    fn get_last_processed(&self) -> Clock;
    fn get_color(&self) -> Color;
    fn get_heat(&self) -> HeatEnergy;
    fn set_heat(&mut self, heat: HeatEnergy) -> Result<(), SetHeatOnZeroHeatCapacityError>;
    fn get_heat_capacity(&self) -> HeatCapacity;
    fn get_density(&self) -> Density;
    fn get_mass(&self, cell_width: f32) -> Mass {
        Mass(self.get_density().0 * cell_width.powi(2))
    }
    fn get_state_of_matter(&self) -> StateOfMatter;
    fn process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        let out = self._process(
            pos,
            coord_dir,
            target_chunk,
            element_grid_conv,
            current_time,
        );
        self._set_last_processed(current_time);
        out
    }
    fn box_clone(&self) -> Box<dyn Element>;

    /// Instructs the loop to swap the element with the element at pos1
    /// you should have already checked to see if pos1 is valid, most likely it comes from another function
    /// as such this function will panic if pos1 is invalid
    fn try_swap_me(
        &self,
        pos1: ConvolutionIdx,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        let mut clone = self.box_clone();
        // Its important we set the last processed time to the current time
        // here because self wont yet have been updated by the process function
        clone._set_last_processed(current_time);
        let prev = element_grid_conv.replace(target_chunk, pos1, clone, current_time);
        match prev {
            Ok(prev) => ElementTakeOptions::ReplaceWith(prev),
            Err(_) => panic!("Tried to swap with an invalid position"),
        }
    }

    // Private elements
    // TODO: Figure out how to make these private
    //       Until then rely on pythonic naming convention
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions;
    fn _set_last_processed(&mut self, current_time: Clock);
}

#[cfg(test)]
mod tests {
    use bevy::render::color::Color;
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
