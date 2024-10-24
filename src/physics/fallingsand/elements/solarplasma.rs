#![expect(missing_docs)]
#![expect(clippy::missing_docs_in_private_items)]
use super::element::{Density, Element, ElementTakeOptions, ElementType, StateOfMatter};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir;

use crate::common::util::clock::Clock;
use crate::common::util::vectors::InChunkJkVector as JkVector;
use bevy::color::palettes::css::ORANGE;
use bevy::color::Color;
use rand::Rng;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct SolarPlasma {
    last_processed: Clock,
}

impl Element for SolarPlasma {
    fn element_type(&self) -> ElementType {
        ElementType::SolarPlasma
    }
    fn density(&self) -> Density {
        Density(100.0)
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
        ORANGE.into()
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
        let below =
            element_grid_conv.idx_below_idx_from_center(target_chunk.coords(), coord_dir, &pos, 1);
        let element = {
            match below {
                Ok(below) => element_grid_conv.get(target_chunk, below),
                Err(err) => Err(err),
            }
        };
        // If we are still in the center chunk, first check if below is vacuum
        // If it is, swap with it
        // Then check if it is water
        // If it is, then either go left or right if they are vacuum
        // Otherwise check if left or right is vacuum
        // If it is, swap with one of them randomly
        match element {
            Ok(element) => {
                if element.state_of_matter() <= StateOfMatter::Gas {
                    self.try_swap_me(
                        below.expect("If Ok(element) then Ok(below)"),
                        target_chunk,
                        element_grid_conv,
                        current_time,
                    )
                } else {
                    let new_idx_l = element_grid_conv.idx_left_right_idx_from_center(
                        target_chunk.coords(),
                        &pos,
                        1,
                    );
                    let new_idx_r = element_grid_conv.idx_left_right_idx_from_center(
                        target_chunk.coords(),
                        &pos,
                        -1,
                    );
                    let element_l = {
                        match new_idx_l {
                            Ok(new_idx_l) => element_grid_conv.get(target_chunk, new_idx_l),
                            Err(err) => Err(err),
                        }
                    };
                    let element_r = {
                        match new_idx_r {
                            Ok(new_idx_r) => element_grid_conv.get(target_chunk, new_idx_r),
                            Err(err) => Err(err),
                        }
                    };

                    // Now decide if we go left or right
                    let mut rng = rand::thread_rng();
                    let rand_bool = rng.gen_bool(0.5);
                    match (element_l, element_r, rand_bool) {
                        (Ok(element_l), Ok(_), false) | (Ok(element_l), Err(_), _) => {
                            if element_l.state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_l.expect("If Ok(element_l) then Ok(new_idx_l)"),
                                    target_chunk,
                                    element_grid_conv,
                                    current_time,
                                )
                            } else {
                                ElementTakeOptions::PutBack
                            }
                        }
                        (Ok(_), Ok(element_r), true) | (Err(_), Ok(element_r), _) => {
                            if element_r.state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_r.expect("If Ok(element_r) then Ok(new_idx_r)"),
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
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
