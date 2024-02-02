use super::element::{
    Density, Element, ElementTakeOptions, ElementType, SetHeatOnZeroHeatCapacityError,
    StateOfMatter,
};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_identifiers::ConvolutionIdentifier;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{Energy, HeatCapacity};
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;
use rand::Rng;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Sand {
    last_processed: Clock,
    heat: Energy,
}

impl Element for Sand {
    fn get_type(&self) -> ElementType {
        ElementType::Sand
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
    fn get_color(&self) -> Color {
        Color::YELLOW
    }
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Go down one cell
        let below = element_grid_conv.get_below_idx_from_center(target_chunk, coord_dir, &pos, 1);
        match below {
            Ok(idx) => {
                match idx.1 {
                    // If we are still in the center chunk, first check if below is vacuum
                    // If it is, swap with it
                    // Otherwise check if left or right is vacuum
                    // If it is, swap with one of them randomly
                    ConvolutionIdentifier::Center => {
                        let element = element_grid_conv.get(target_chunk, idx);
                        match element {
                            Ok(element) => {
                                if element.get_state_of_matter() <= StateOfMatter::Liquid {
                                    self.try_swap_me(
                                        idx,
                                        target_chunk,
                                        element_grid_conv,
                                        current_time,
                                    )
                                } else {
                                    let new_idx_l = element_grid_conv
                                        .get_left_right_idx_from_center(target_chunk, &idx.0, 1);
                                    let new_idx_r = element_grid_conv
                                        .get_left_right_idx_from_center(target_chunk, &idx.0, -1);
                                    let element_l = {
                                        match new_idx_l {
                                            Ok(new_idx_l) => {
                                                element_grid_conv.get(target_chunk, new_idx_l)
                                            }
                                            Err(err) => Err(err),
                                        }
                                    };
                                    let element_r = {
                                        match new_idx_r {
                                            Ok(new_idx_r) => {
                                                element_grid_conv.get(target_chunk, new_idx_r)
                                            }
                                            Err(err) => Err(err),
                                        }
                                    };

                                    // Now decide if we go left or right
                                    let mut rng = rand::thread_rng();
                                    let rand_bool = rng.gen_bool(0.5);
                                    match (element_l, element_r, rand_bool) {
                                        (Ok(element_l), Ok(_), false) => {
                                            if element_l.get_state_of_matter()
                                                <= StateOfMatter::Liquid
                                            {
                                                self.try_swap_me(
                                                    new_idx_l.unwrap(),
                                                    target_chunk,
                                                    element_grid_conv,
                                                    current_time,
                                                )
                                            } else {
                                                ElementTakeOptions::PutBack
                                            }
                                        }
                                        (Ok(_), Ok(element_r), true) => {
                                            if element_r.get_state_of_matter()
                                                <= StateOfMatter::Liquid
                                            {
                                                self.try_swap_me(
                                                    new_idx_r.unwrap(),
                                                    target_chunk,
                                                    element_grid_conv,
                                                    current_time,
                                                )
                                            } else {
                                                ElementTakeOptions::PutBack
                                            }
                                        }
                                        (Ok(element_l), Err(_), _) => {
                                            if element_l.get_state_of_matter()
                                                <= StateOfMatter::Liquid
                                            {
                                                self.try_swap_me(
                                                    new_idx_l.unwrap(),
                                                    target_chunk,
                                                    element_grid_conv,
                                                    current_time,
                                                )
                                            } else {
                                                ElementTakeOptions::PutBack
                                            }
                                        }
                                        (Err(_), Ok(element_r), _) => {
                                            if element_r.get_state_of_matter()
                                                <= StateOfMatter::Liquid
                                            {
                                                self.try_swap_me(
                                                    new_idx_r.unwrap(),
                                                    target_chunk,
                                                    element_grid_conv,
                                                    current_time,
                                                )
                                            } else {
                                                ElementTakeOptions::PutBack
                                            }
                                        }
                                        (Err(_), Err(_), _) => ElementTakeOptions::PutBack,
                                    }
                                }
                            }
                            Err(_) => ElementTakeOptions::PutBack,
                        }
                    }
                    _ => {
                        // If we would not in the center chunk anymore, just swap with below
                        let element = element_grid_conv.get(target_chunk, idx);
                        match element {
                            Ok(element) => {
                                if element.get_state_of_matter() <= StateOfMatter::Liquid {
                                    self.try_swap_me(
                                        idx,
                                        target_chunk,
                                        element_grid_conv,
                                        current_time,
                                    )
                                } else {
                                    ElementTakeOptions::PutBack
                                }
                            }
                            Err(_) => ElementTakeOptions::PutBack,
                        }
                    }
                }
            }
            Err(_) => ElementTakeOptions::PutBack,
        }
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }

    fn get_heat(&self) -> Energy {
        self.heat
    }

    fn get_heat_capacity(&self) -> HeatCapacity {
        HeatCapacity(1.0)
    }

    fn set_heat(&mut self, heat: Energy) -> Result<(), SetHeatOnZeroHeatCapacityError> {
        self.heat = heat;
        Ok(())
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

        fn assert_movement(
            mut element_grid_dir: ElementGridDir,
            loc1: (ChunkIjkVector, JkVector),
            loc2: (ChunkIjkVector, JkVector),
        ) {
            let mut clock = Clock::default();

            // Set the bottom right to sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let sand = Sand::default();
                chunk.set(loc1.1, Box::new(sand), clock);
            }

            // Now process one frame
            clock.update(Duration::from_millis(100));
            element_grid_dir.process_full(clock);

            // Now check that this chunk location no longer has sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let previous_location_type = chunk.get(loc1.1).get_type();
                assert_ne!(previous_location_type, ElementType::Sand);
            }

            // Now check that the chunk below has sand
            {
                let below_chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc2.0);
                let below_location_type = below_chunk.get(loc2.1).get_type();
                assert_eq!(below_location_type, ElementType::Sand);
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

        test_movement!(test_movement_i2_j2_k1, (2, 2, 1), (2, 1, 1));
    }
}
