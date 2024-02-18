use super::element::{Density, Element, ElementTakeOptions, ElementType, StateOfMatter};
use super::movement::fluid::fluid_process;
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;

use crate::physics::util::clock::Clock;
use bevy::render::color::Color;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Water {
    last_processed: Clock,
}

impl Element for Water {
    fn get_type(&self) -> ElementType {
        ElementType::Water
    }
    fn get_density(&self) -> Density {
        Density(1.0)
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    fn get_state_of_matter(&self) -> StateOfMatter {
        StateOfMatter::Liquid
    }
    fn get_color(&self) -> Color {
        Color::BLUE
    }
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        fluid_process(
            self,
            pos,
            coord_dir,
            target_chunk,
            element_grid_conv,
            current_time,
        )
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
