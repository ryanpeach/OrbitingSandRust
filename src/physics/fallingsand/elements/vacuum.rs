use bevy::log::error;
use bevy::render::color::Color;

use super::element::{
    Density, Element, ElementTakeOptions, ElementType, SetHeatOnZeroHeatCapacityError,
    StateOfMatter,
};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{Energy, HeatCapacity};
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
        Color::BLACK
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

    fn get_heat(&self) -> Energy {
        Energy(0.0)
    }
    fn set_heat(&mut self, heat: Energy) -> Result<(), SetHeatOnZeroHeatCapacityError> {
        Err(SetHeatOnZeroHeatCapacityError)
    }

    fn get_heat_capacity(&self) -> HeatCapacity {
        HeatCapacity(0.0)
    }
}
