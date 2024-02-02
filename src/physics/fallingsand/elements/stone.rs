use super::element::{
    Density, Element, ElementTakeOptions, ElementType, SetHeatOnZeroHeatCapacityError,
    StateOfMatter,
};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{HeatCapacity, HeatEnergy};
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Stone {
    last_processed: Clock,
    heat: HeatEnergy,
}

impl Element for Stone {
    fn get_type(&self) -> ElementType {
        ElementType::Stone
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
        StateOfMatter::Solid
    }
    // Gray
    fn get_color(&self) -> Color {
        Color::rgb_u8(128, 128, 128)
    }
    // Stone does nothing
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

    fn get_heat(&self) -> HeatEnergy {
        self.heat
    }
    fn set_heat(&mut self, heat: HeatEnergy) -> Result<(), SetHeatOnZeroHeatCapacityError> {
        self.heat = heat;
        Ok(())
    }

    fn get_heat_capacity(&self) -> HeatCapacity {
        HeatCapacity(1.0)
    }
}

// 6, 0, 0
#[cfg(test)]
mod tests {
    use crate::physics::fallingsand::{
        data::element_directory::ElementGridDir, mesh::coordinate_directory::CoordinateDirBuilder,
    };

    use super::*;

    /// The default element grid directory for testing
    fn get_element_grid_dir() -> ElementGridDir {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(10)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();
        ElementGridDir::new_empty(coordinate_dir)
    }

    /// Simple tests for testing that the sand falls down
    mod falls_down {
        use std::time::Duration;

        use super::*;
        use crate::physics::fallingsand::{
            elements::element::ElementType,
            util::vectors::{ChunkIjkVector, IjkVector, JkVector},
        };

        fn assert_movement(mut element_grid_dir: ElementGridDir, loc1: (ChunkIjkVector, JkVector)) {
            let mut clock = Clock::default();

            // Set the bottom right to sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let sand = Stone::default();
                chunk.set(loc1.1, Box::new(sand), clock);
            }

            // Now process one frame
            clock.update(Duration::from_millis(100));
            element_grid_dir.process_full(clock);

            // Now check that the chunk still has stone
            {
                let below_chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let below_location_type = below_chunk.get(loc1.1).get_type();
                assert_eq!(below_location_type, ElementType::Stone);
            }
        }

        macro_rules! test_movement {
            ($name:ident, $pos1:expr) => {
                #[test]
                fn $name() {
                    let element_grid_dir = get_element_grid_dir();
                    let pos1 = element_grid_dir
                        .get_coordinate_dir()
                        .cell_idx_to_chunk_idx(IjkVector::new($pos1.0, $pos1.1, $pos1.2));
                    assert_movement(element_grid_dir, pos1);
                }
            };
        }

        test_movement!(test_no_movement_i2_j2_k1, (2, 2, 1));
    }
}
