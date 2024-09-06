use bevy::color::palettes::css::BLUE;
use bevy::color::Color;

use super::element::{Density, Element, ElementTakeOptions, ElementType, StateOfMatter};
use super::movement::fluid::fluid_process;
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;

use crate::physics::util::clock::Clock;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Water {
    last_processed: Clock,
}

impl Element for Water {
    fn element_type(&self) -> ElementType {
        ElementType::Water
    }
    fn density(&self) -> Density {
        Density(1.0)
    }
    fn last_processed(&self) -> Clock {
        self.last_processed
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    fn state_of_matter(&self) -> StateOfMatter {
        StateOfMatter::Liquid
    }
    fn color(&self) -> Color {
        BLUE.into()
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
