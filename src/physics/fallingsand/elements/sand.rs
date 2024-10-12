#![expect(missing_docs)]
#![expect(clippy::missing_docs_in_private_items)]
use bevy::color::palettes::css::YELLOW;
use bevy::color::Color;

use super::element::{Density, Element, ElementTakeOptions, ElementType, StateOfMatter};
use super::movement::solid::solid_process;
use crate::common::util::clock::Clock;
use crate::common::util::vectors::{ChunkIjkVector, ChunkJkVector, InChunkJkVector as JkVector};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;

use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Sand {
    last_processed: Clock,
}

impl Element for Sand {
    fn element_type(&self) -> ElementType {
        ElementType::Sand
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
        StateOfMatter::Solid
    }
    fn color(&self) -> Color {
        YELLOW.into()
    }
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        solid_process(
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

// 6, 0, 0
#[cfg(test)]
mod tests {
    #![allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::unwrap_used,
        clippy::panic
    )]
    use crate::common::util::uom::Length;
    use crate::physics::fallingsand::{
        data::element_directory::ElementGridDir, mesh::coordinate_dir::Builder,
    };

    use super::*;

    /// The default element grid directory for testing
    fn element_grid_dir() -> ElementGridDir {
        let coordinate_dir = Builder::new()
            .cell_width(Length(1.0))
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
        use crate::common::util::vectors::{
            ChunkIjkVector, ChunkJkVector, FullIdx, IjkVector, InChunkJkVector as JkVector,
        };
        use crate::physics::fallingsand::elements::element::ElementType;

        fn assert_movement(mut element_grid_dir: ElementGridDir, loc1: FullIdx, loc2: FullIdx) {
            let mut clock = Clock::default();

            // Set the bottom right to sand
            {
                let chunk = element_grid_dir.chunk_at_chunk_ijk_mut(loc1.chunk_idx);
                let sand = Sand::default();
                chunk.set(loc1.pos, Box::new(sand), clock);
            }

            // Now process one frame
            clock.update(Duration::from_millis(100));
            element_grid_dir.process_single_chunk(clock, loc1.chunk_idx);

            // Now check that this chunk location no longer has sand
            {
                let chunk = element_grid_dir.chunk_at_chunk_ijk_mut(loc1.chunk_idx);
                let previous_location_type = chunk.get(loc1.pos).element_type();
                assert_ne!(
                    previous_location_type,
                    ElementType::Sand,
                    "Previous location {loc1:?} still has a downflier"
                );
            }

            // Now check that the chunk below has sand
            {
                let below_chunk = element_grid_dir.chunk_at_chunk_ijk_mut(loc2.chunk_idx);
                let below_location_type = below_chunk.get(loc2.pos).element_type();
                assert_eq!(
                    below_location_type,
                    ElementType::Sand,
                    "New location {loc2:?} does not have a downflier"
                );
            }
        }

        macro_rules! test_movement {
            ($name:ident, $pos1:expr, $pos2:expr) => {
                #[test]
                fn $name() {
                    let element_grid_dir = element_grid_dir();
                    let pos1 = element_grid_dir
                        .coordinate_dir()
                        .cell_idx_to_full_idx(IjkVector::new($pos1.0, $pos1.1, $pos1.2));
                    let pos2 = element_grid_dir
                        .coordinate_dir()
                        .cell_idx_to_full_idx(IjkVector::new($pos2.0, $pos2.1, $pos2.2));
                    assert_movement(element_grid_dir, pos1, pos2);
                }
            };
        }

        test_movement!(test_movement_i2_j2_k1, (2, 2, 1), (2, 1, 1));
    }
}
