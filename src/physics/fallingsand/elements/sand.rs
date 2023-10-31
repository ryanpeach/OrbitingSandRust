use super::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_grids::ConvOutOfBoundsError;
use crate::physics::fallingsand::convolution::neighbor_identifiers::{
    ConvolutionIdentifier, ConvolutionIdx,
};
use crate::physics::fallingsand::coordinates::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;

pub fn get_bottom_left(
    conv: &ElementGridConvolutionNeighbors,
    target_grid: &ElementGrid,
    coord_dir: &CoordinateDir,
    pos: &JkVector,
    n: usize,
) -> Result<(ConvolutionIdx, Box<dyn Element>), ConvOutOfBoundsError> {
    let idx = conv.get_below_idx_from_center(target_grid, coord_dir, pos, n)?;
    match idx.1 {
        ConvolutionIdentifier::Bottom(bottom_id) => {
            let new_idx = conv.get_left_right_idx_from_bottom_center(&idx.0, bottom_id, 1)?;
            Ok((new_idx, conv.get(target_grid, new_idx)?.box_clone()))
        }
        ConvolutionIdentifier::Center => {
            let new_idx = conv.get_left_right_idx_from_center(target_grid, &idx.0, 1)?;
            Ok((new_idx, conv.get(target_grid, new_idx)?.box_clone()))
        }
        _ => panic!("get_below_idx_from_center returned an invalid index"),
    }
}

pub fn get_bottom_right(
    conv: &ElementGridConvolutionNeighbors,
    target_grid: &ElementGrid,
    coord_dir: &CoordinateDir,
    pos: &JkVector,
    n: usize,
) -> Result<(ConvolutionIdx, Box<dyn Element>), ConvOutOfBoundsError> {
    let idx = conv.get_below_idx_from_center(target_grid, coord_dir, pos, n)?;
    match idx.1 {
        ConvolutionIdentifier::Bottom(bottom_id) => {
            let new_idx = conv.get_left_right_idx_from_bottom_center(&idx.0, bottom_id, -1)?;
            Ok((new_idx, conv.get(target_grid, new_idx)?.box_clone()))
        }
        ConvolutionIdentifier::Center => {
            let new_idx = conv.get_left_right_idx_from_center(target_grid, &idx.0, -1)?;
            Ok((new_idx, conv.get(target_grid, new_idx)?.box_clone()))
        }
        _ => panic!("get_below_idx_from_center returned an invalid index"),
    }
}

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Sand {
    last_processed: Clock,
}

impl Element for Sand {
    fn get_type(&self) -> ElementType {
        ElementType::Sand
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    #[allow(clippy::borrowed_box)]
    fn get_color(&self) -> Color {
        Color::YELLOW
    }
    fn process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Doing this as a way to make sure I set last_processed AFTER I've done all the processing
        let out: ElementTakeOptions = {
            let below =
                element_grid_conv.get_below_idx_from_center(target_chunk, coord_dir, &pos, 1);
            match below {
                Ok(idx) => {
                    if let Ok(element) = element_grid_conv.get(target_chunk, idx) {
                        match element.get_type() {
                            ElementType::Vacuum => {
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
        self.last_processed = current_time;
        out
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}

// 6, 0, 0
#[cfg(test)]
mod tests {
    use crate::physics::fallingsand::{
        coordinates::coordinate_directory::CoordinateDirBuilder, element_directory::ElementGridDir,
    };

    use super::*;

    /// The default element grid directory for testing
    fn get_element_grid_dir() -> ElementGridDir {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(10)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_cells(64 * 64)
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
            let mut clock = Clock::new();

            // Set the bottom right to sand
            {
                let chunk = element_grid_dir.get_chunk_by_chunk_ijk_mut(loc1.0);
                let sand = Sand::default();
                chunk.set(loc1.1, Box::new(sand), clock);
            }

            // Now process one frame
            for _ in 0..9 {
                clock.update(Duration::from_millis(100));
                element_grid_dir.process(clock);
            }

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
