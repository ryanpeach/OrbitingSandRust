use bevy::render::color::Color;

use super::element::{
    Compressability, Density, Element, ElementTakeOptions, ElementType,
    SetHeatOnZeroSpecificHeatError, StateOfMatter,
};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{
    HeatEnergy, SpecificHeat, ThermalConductivity, ThermodynamicTemperature,
};
use crate::physics::util::clock::Clock;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Vacuum {
    last_processed: Clock,
}

impl Element for Vacuum {
    fn get_type(&self) -> ElementType {
        ElementType::Vacuum
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn get_density(&self) -> Density {
        Density(0.0)
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    fn get_state_of_matter(&self) -> StateOfMatter {
        StateOfMatter::Empty
    }
    fn get_color(&self) -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }
    fn _process(
        &mut self,
        _pos: JkVector,
        _coord_dir: &CoordinateDir,
        _target_chunk: &mut ElementGrid,
        _element_grid_conv: &mut ElementGridConvolutionNeighbors,
        _current_time: Clock,
    ) -> ElementTakeOptions {
        ElementTakeOptions::PutBack
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }

    fn get_default_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature(0.0)
    }

    fn get_heat(&self) -> HeatEnergy {
        HeatEnergy(0.0)
    }

    fn set_heat(
        &mut self,
        _heat: HeatEnergy,
        _current_time: Clock,
    ) -> Result<(), SetHeatOnZeroSpecificHeatError> {
        Err(SetHeatOnZeroSpecificHeatError)
    }

    fn get_specific_heat(&self) -> SpecificHeat {
        SpecificHeat(0.0)
    }
    fn get_thermal_conductivity(&self) -> ThermalConductivity {
        ThermalConductivity(0.0)
    }

    fn get_compressability(&self) -> Compressability {
        Compressability(0.0)
    }
}
