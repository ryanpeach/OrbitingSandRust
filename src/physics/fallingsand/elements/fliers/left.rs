use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::elements::element::{
    Element, ElementTakeOptions, ElementType, StateOfMatter,
};
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct LeftFlier {
    last_processed: Clock,
}

impl Element for LeftFlier {
    fn get_type(&self) -> ElementType {
        ElementType::LeftFlier
    }
    fn get_mass(&self) -> f32 {
        0.0
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
    fn get_color(&self) -> Color {
        Color::rgb_u8(254, 254, 254)
    }
    fn _process(
        &mut self,
        pos: JkVector,
        _coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Doing this as a way to make sure I set last_processed AFTER I've done all the processing
        let out: ElementTakeOptions = {
            let left = element_grid_conv.get_left_right_idx_from_center(target_chunk, &pos, 1);
            match left {
                Ok(idx) => {
                    if let Ok(element) = element_grid_conv.get(target_chunk, idx) {
                        match element.get_state_of_matter() {
                            StateOfMatter::Empty => {
                                self.try_swap_me(idx, target_chunk, element_grid_conv, current_time)
                            }
                            _ => ElementTakeOptions::PutBack,
                        }
                    } else {
                        ElementTakeOptions::PutBack
                    }
                }
                Err(_) => ElementTakeOptions::PutBack,
            }
        };
        out
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}

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
            .num_layers(7)
            .first_num_radial_lines(12)
            .second_num_concentric_circles(3)
            .first_num_radial_chunks(3)
            .max_radial_lines_per_chunk(128)
            .max_concentric_circles_per_chunk(128)
            .build();
        ElementGridDir::new_empty(coordinate_dir)
    }

    /// Simple tests for testing that the sand falls down
    mod moves {
        use std::time::Duration;

        use super::*;
        use crate::physics::fallingsand::{
            elements::element::ElementType,
            util::vectors::{ChunkIjkVector, IjkVector, JkVector},
        };

        fn assert_movement(
            mut element_grid_dir: ElementGridDir,
            loc1: (ChunkIjkVector, JkVector),
            loc2: (ChunkIjkVector, JkVector),
        ) {
            let mut clock = Clock::default();

            // Set the bottom right to sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let sand = LeftFlier::default();
                chunk.set(loc1.1, Box::new(sand), clock);
            }

            // Now process one frame
            clock.update(Duration::from_millis(100));
            element_grid_dir.process_single_chunk(clock, loc1.0);

            // Now check that this chunk location no longer has sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let previous_location_type = chunk.get(loc1.1).get_type();
                assert_ne!(
                    previous_location_type,
                    ElementType::LeftFlier,
                    "Previous location {:?} still has a leftflier",
                    loc1
                );
            }

            // Now check that the chunk below has sand
            {
                let left_chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc2.0);
                let left_location_type = left_chunk.get(loc2.1).get_type();
                assert_eq!(
                    left_location_type,
                    ElementType::LeftFlier,
                    "New location {:?} does not have a leftflier",
                    loc2
                );
            }
        }

        macro_rules! test_movement {
            ($name:ident, $pos1:expr, $pos2:expr) => {
                #[test]
                fn $name() {
                    let element_grid_dir = get_element_grid_dir();
                    let pos1 = element_grid_dir
                        .get_coordinate_dir()
                        .cell_idx_to_chunk_idx(IjkVector::new($pos1.0, $pos1.1, $pos1.2));
                    let pos2 = element_grid_dir
                        .get_coordinate_dir()
                        .cell_idx_to_chunk_idx(IjkVector::new($pos2.0, $pos2.1, $pos2.2));
                    assert_movement(element_grid_dir, pos1, pos2);
                }
            };
        }

        test_movement!(test_movement_i2_j0_k31_left, (2, 0, 31), (2, 0, 32));

        test_movement!(test_movement_i2_j0_k47_left, (2, 0, 47), (2, 0, 0));

        test_movement!(test_movement_i2_j0_k1_left, (2, 0, 1), (2, 0, 2));
    }
}
