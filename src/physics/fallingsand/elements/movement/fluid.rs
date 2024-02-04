use rand::Rng;

use crate::physics::{
    fallingsand::{
        convolution::behaviors::ElementGridConvolutionNeighbors,
        data::element_grid::ElementGrid,
        elements::element::{Element, ElementTakeOptions, StateOfMatter},
        mesh::coordinate_directory::CoordinateDir,
        util::vectors::JkVector,
    },
    util::clock::Clock,
};

pub fn fluid_process(
    self_element: &mut dyn Element,
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
            if element.get_state_of_matter() <= StateOfMatter::Gas {
                self_element.try_swap_me(
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
                        if element_l.get_state_of_matter() <= StateOfMatter::Gas {
                            self_element.try_swap_me(
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
                        if element_r.get_state_of_matter() <= StateOfMatter::Gas {
                            self_element.try_swap_me(
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
                        if element_l.get_state_of_matter() <= StateOfMatter::Gas {
                            self_element.try_swap_me(
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
                        if element_r.get_state_of_matter() <= StateOfMatter::Gas {
                            self_element.try_swap_me(
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
