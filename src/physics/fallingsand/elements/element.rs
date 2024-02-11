//! This module contains the trait Element and associated types
//! This is the trait that all elements must implement
//! It also contains info about states of matter and other useful enums and components

use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_identifiers::ConvolutionIdx;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{
    Compressability, Density, HeatEnergy, Length, SpecificHeat, ThermalConductivity,
    ThermodynamicTemperature,
};
use crate::physics::orbits::components::{Mass};
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;
use strum_macros::EnumIter;

use super::fliers::down::DownFlier;
use super::fliers::left::LeftFlier;
use super::fliers::right::RightFlier;
use super::lava::Lava;
use super::sand::Sand;
use super::solarplasma::SolarPlasma;
use super::stone::Stone;
use super::vacuum::Vacuum;
use super::water::Water;

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

/// Allows you to match on the type of element
/// each element impl has a unique item in this enum
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum ElementType {
    #[default]
    Vacuum,
    Sand,
    Stone,
    Lava,
    Water,
    SolarPlasma,
    DownFlier,
    LeftFlier,
    RightFlier,
}

impl ElementType {
    /// This gets the default element of the type
    pub fn get_element(&self, cell_width: Length) -> Box<dyn Element> {
        match self {
            ElementType::Vacuum => Box::<Vacuum>::default(),
            ElementType::DownFlier => Box::<DownFlier>::default(),
            ElementType::LeftFlier => Box::<LeftFlier>::default(),
            ElementType::RightFlier => Box::<RightFlier>::default(),
            ElementType::Sand => Box::<Sand>::new(Sand::new(cell_width)),
            ElementType::Stone => Box::<Stone>::new(Stone::new(cell_width)),
            ElementType::Water => Box::<Water>::new(Water::new(cell_width)),
            ElementType::SolarPlasma => Box::<SolarPlasma>::new(SolarPlasma::new(cell_width)),
            ElementType::Lava => Box::<Lava>::new(Lava::new(cell_width)),
        }
    }
}

/// If something has 0 heat capacity or specific heat, you should not set its heat
#[derive(Default, Debug)]
pub struct SetHeatOnZeroSpecificHeatError;

/// This is the trait that all elements must implement
pub trait Element: Send + Sync {
    /// This gets the type of the element
    /// Converts between the trait and the enum
    fn get_type(&self) -> ElementType;
    /// This gets the last time the element was processed
    /// Useful for physics calculations by getting the dt between now and then
    fn get_last_processed(&self) -> Clock;
    /// This gets the color of the element
    /// Always constant and unique for each element, so that we can process them
    /// in fragment shaders knowing their type just by their color
    /// You can map them to other colors and add effects using the fragment shader
    fn get_color(&self) -> Color;
    /// This gets the heat of the element
    fn get_heat(&self) -> HeatEnergy;
    /// This sets the heat of the element
    /// Do not call this if the heat capacity is 0
    fn set_heat(
        &mut self,
        heat: HeatEnergy,
        current_time: Clock,
    ) -> Result<(), SetHeatOnZeroSpecificHeatError>;
    /// This gets the specific heat capacity of the element at atp
    /// Usually constant
    fn get_specific_heat(&self) -> SpecificHeat;
    /// This gets the thermal conductivity of the element at atp
    /// Usually constant
    /// Source: <https://www.engineeringtoolbox.com/thermal-conductivity-d_429.html>
    fn get_thermal_conductivity(&self) -> ThermalConductivity;
    /// This is a convienence function that gets the "default" temperature of an element
    /// For example, lava should start out hot, ice cold, etc.
    /// This answers the question "how hot is the element when it is created?"
    /// Usually constant
    fn get_default_temperature(&self) -> ThermodynamicTemperature;
    /// Get the actual temperature of the element
    fn get_temperature(&self, cell_width: Length) -> ThermodynamicTemperature {
        self.get_heat().temperature(
            self.get_specific_heat()
                .heat_capacity(self.get_mass(cell_width)),
        )
    }
    /// This gets the density of the element relative to the cell_width
    /// This is so bigger cells have more mass, so we don't have to have as many cells
    /// for simpler bodies, like gas giants or the sun
    fn get_density(&self) -> Density;
    /// This gets the compressibility of the element under pressure
    /// Usually constant
    fn get_compressability(&self) -> Compressability;
    /// This gets the mass of the element based on the density and the cell_width
    fn get_mass(&self, cell_width: Length) -> Mass {
        self.get_density().mass(cell_width)
    }
    /// This gets the state of matter of the element
    fn get_state_of_matter(&self) -> StateOfMatter;
    /// This is the "public" process method, that calls the private _process method
    /// makes sure that _set_last_processed is called
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
    /// This is the way we implement clone for a trait object
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

    /// This is the private process method to be implemented by the element
    /// Takes in the position of the element in the grid
    /// The coordinate grid which gives us information about the celestial body
    /// The target chunk which is the element grid this element was a part of
    /// The element grid convolution neighbors which gives you the ability to move
    ///    and look around at neighboring chunks
    /// The current time
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions;

    /// Set the last time the element was processed
    /// No need to call this publicly, it is called by the public process method
    fn _set_last_processed(&mut self, current_time: Clock);
}

#[cfg(test)]
mod tests {
    use bevy::render::color::Color;
    use strum::IntoEnumIterator;

    use crate::physics::heat::components::Length;

    use super::ElementType;

    /// This tests that all elements have different colors
    /// This is important because we use the color to identify the element in shaders
    #[test]
    fn test_all_elements_have_different_color() {
        let mut colors = Vec::<Color>::new();
        for element_type in ElementType::iter() {
            let color = element_type.get_element(Length(1.0)).get_color();
            assert!(
                !colors.contains(&color),
                "Color {:?} of element {:?} is not unique",
                color,
                element_type
            );
            colors.push(color);
        }
    }

    /// This tests that all elements have different types
    #[test]
    fn test_all_elements_have_different_type() {
        let mut types = Vec::<ElementType>::new();
        for element_type in ElementType::iter() {
            assert!(
                !types.contains(&element_type),
                "Element type {:?} is not unique",
                element_type
            );
            types.push(element_type);
        }
    }

    /// This tests that all enums and elements refer to each other
    #[test]
    fn test_all_types_and_elements_correspond() {
        for element_type in ElementType::iter() {
            let element = element_type.get_element(Length(1.0));
            assert_eq!(
                element_type,
                element.get_type(),
                "Element type {:?} does not match the type of the element",
                element_type
            );
        }
    }
}
