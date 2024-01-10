use super::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;
use rand::Rng;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct Water {
    last_processed: Clock,
}

impl Element for Water {
    fn get_type(&self) -> ElementType {
        ElementType::Water
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    #[allow(clippy::borrowed_box)]
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
        // Go down one cell
        let below = element_grid_conv.get_below_idx_from_center(target_chunk, coord_dir, &pos, 1);
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
                if element.get_type() == ElementType::Vacuum {
                    self.try_swap_me(
                        below.unwrap(),
                        target_chunk,
                        element_grid_conv,
                        current_time,
                    )
                } else {
                    let new_idx_l =
                        element_grid_conv.get_left_right_idx_from_center(target_chunk, &pos, 1);
                    let new_idx_r =
                        element_grid_conv.get_left_right_idx_from_center(target_chunk, &pos, -1);
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
                        (Ok(element_l), Ok(_), false) => {
                            if element_l.get_type() == ElementType::Vacuum {
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
                            if element_r.get_type() == ElementType::Vacuum {
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
                            if element_l.get_type() == ElementType::Vacuum {
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
                            if element_r.get_type() == ElementType::Vacuum {
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
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
