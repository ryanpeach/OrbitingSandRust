use super::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::convolution::neighbor_grids::ConvOutOfBoundsError;
use crate::physics::fallingsand::convolution::neighbor_identifiers::{
    ConvolutionIdentifier, ConvolutionIdx,
};
use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::element_grid::ElementGrid;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;

pub fn get_bottom_left(
    conv: &ElementGridConvolutionNeighbors,
    target_grid: &ElementGrid,
    pos: &JkVector,
    n: usize,
) -> Result<(ConvolutionIdx, Box<dyn Element>), ConvOutOfBoundsError> {
    let idx = conv.get_below_idx_from_center(target_grid, pos, n)?;
    match idx.1 {
        ConvolutionIdentifier::Bottom(bottom_id) => {
            let new_idx = conv.get_left_right_idx_from_bottom(&idx.0, bottom_id, 1)?;
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
    pos: &JkVector,
    n: usize,
) -> Result<(ConvolutionIdx, Box<dyn Element>), ConvOutOfBoundsError> {
    let idx = conv.get_below_idx_from_center(target_grid, pos, n)?;
    match idx.1 {
        ConvolutionIdentifier::Bottom(bottom_id) => {
            let new_idx = conv.get_left_right_idx_from_bottom(&idx.0, bottom_id, -1)?;
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
    fn get_color(&self, _pos: JkVector, _chunk_coords: &Box<dyn ChunkCoords>) -> Color {
        Color::YELLOW
    }
    fn process(
        &mut self,
        pos: JkVector,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Doing this as a way to make sure I set last_processed AFTER I've done all the processing
        let out: ElementTakeOptions = {
            let below = element_grid_conv.get_below_idx_from_center(target_chunk, &pos, 1);
            match below {
                Ok(idx) => {
                    if let Ok(element) = element_grid_conv.get(target_chunk, idx) {
                        match element.get_type() {
                            ElementType::Vacuum => self.try_swap_me(
                                pos,
                                idx,
                                target_chunk,
                                element_grid_conv,
                                current_time,
                            ),
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
