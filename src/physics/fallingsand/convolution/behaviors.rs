use std::fmt;

use hashbrown::HashMap;

use crate::physics::{
    fallingsand::{
        element_grid::ElementGrid,
        elements::element::Element,
        util::{
            functions::modulo,
            vectors::{ChunkIjkVector, JkVector},
        },
    },
    util::clock::Clock,
};

use super::{
    neighbor_grids::{
        BottomNeighborGrids, ConvOutOfBoundsError, ElementGridConvolutionNeighborGrids,
        LeftRightNeighborGrids, TopNeighborGrids,
    },
    neighbor_identifiers::{
        BottomNeighborIdentifier, BottomNeighborIdentifierLayerTransition,
        BottomNeighborIdentifierNormal, ConvolutionIdentifier, ConvolutionIdx,
        LeftRightNeighborIdentifier, LeftRightNeighborIdentifierLR, TopNeighborIdentifier,
        TopNeighborIdentifierLayerTransition, TopNeighborIdentifierNormal,
    },
    neighbor_indexes::{
        BottomNeighborIdxs, ElementGridConvolutionNeighborIdxs,
        ElementGridConvolutionNeighborIdxsIter, LeftRightNeighborIdxs,
    },
};

pub struct ElementGridConvolutionNeighbors {
    chunk_idxs: ElementGridConvolutionNeighborIdxs,
    grids: ElementGridConvolutionNeighborGrids,
}

/// Instantiation
impl ElementGridConvolutionNeighbors {
    pub fn new(
        chunk_idxs: ElementGridConvolutionNeighborIdxs,
        mut grids: HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        let lr_neighbors = LeftRightNeighborGrids::from_hashmap(&chunk_idxs.left_right, &mut grids);
        let top_neighbors = TopNeighborGrids::from_hashmap(&chunk_idxs.top, &mut grids);
        let bottom_neighbors = BottomNeighborGrids::from_hashmap(&chunk_idxs.bottom, &mut grids);
        ElementGridConvolutionNeighbors {
            chunk_idxs: chunk_idxs,
            grids: ElementGridConvolutionNeighborGrids {
                left_right: lr_neighbors,
                top: top_neighbors,
                bottom: bottom_neighbors,
            },
        }
    }
}

/// Iteration
/// We are going to implement into interation on the Neighbors so that unpackaging is easier
/// To do this we will use the into_hashmap method on the neighbor grids
/// and the iter method on the neighbor indexes
/// taking from the hashmap on each iteration of the iter
pub struct ElementGridConvolutionNeighborsIter {
    chunk_idxs_iter: ElementGridConvolutionNeighborIdxsIter,
    grids: HashMap<ChunkIjkVector, ElementGrid>,
}

impl Iterator for ElementGridConvolutionNeighborsIter {
    type Item = (ChunkIjkVector, ElementGrid);
    fn next(&mut self) -> Option<Self::Item> {
        match self.chunk_idxs_iter.next() {
            Some(chunk_idx) => {
                let grid = self.grids.remove(&chunk_idx)?;
                Some((chunk_idx, grid))
            }
            None => None,
        }
    }
}

impl IntoIterator for ElementGridConvolutionNeighbors {
    type Item = (ChunkIjkVector, ElementGrid);
    type IntoIter = ElementGridConvolutionNeighborsIter;
    fn into_iter(self) -> Self::IntoIter {
        ElementGridConvolutionNeighborsIter {
            chunk_idxs_iter: self.chunk_idxs.iter(),
            grids: self.grids.into_hashmap(),
        }
    }
}

/// Coordinate tranformations
/// Methods which allow you to get an index "relative" to another index on the element convolution.
impl ElementGridConvolutionNeighbors {
    /// Gets the element n cells below the position in the target chunk
    /// the target chunk is the chunk in the center of the convolution
    /// the pos is the position in the target chunk
    pub fn get_below_idx_from_center(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
        n: usize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        let b_concentric_circles = self.grids.bottom.get_num_concentric_circles();
        let (new_j, new_k) = if pos.j >= n {
            (pos.j - n, pos.k)
        } else if pos.j as isize - n as isize + b_concentric_circles as isize >= 0 {
            (pos.j + b_concentric_circles - n, pos.k / 2)
        } else {
            return Err(ConvOutOfBoundsError(ConvolutionIdx(
                JkVector {
                    j: pos.j - n,
                    k: pos.k,
                },
                ConvolutionIdentifier::Center,
            )));
        };

        let new_coords = JkVector { j: new_j, k: new_k };

        let conv_id = match self.chunk_idxs.bottom {
            BottomNeighborIdxs::BottomOfGrid if pos.j == 0 => Err(ConvOutOfBoundsError(
                ConvolutionIdx(new_coords, ConvolutionIdentifier::Center),
            )),
            BottomNeighborIdxs::FullLayerBelow { .. } => Ok(ConvolutionIdentifier::Bottom(
                BottomNeighborIdentifier::FullLayerBelow,
            )),
            BottomNeighborIdxs::LayerTransition { .. } => {
                let transition = if target_chunk.get_chunk_coords().get_chunk_idx().k % 2 == 0 {
                    BottomNeighborIdentifierLayerTransition::BottomLeft
                } else {
                    BottomNeighborIdentifierLayerTransition::BottomRight
                };
                Ok(ConvolutionIdentifier::Bottom(
                    BottomNeighborIdentifier::LayerTransition(transition),
                ))
            }
            BottomNeighborIdxs::Normal { .. } => Ok(ConvolutionIdentifier::Bottom(
                BottomNeighborIdentifier::Normal(BottomNeighborIdentifierNormal::Bottom),
            )),
            _ => return Ok(ConvolutionIdx(new_coords, ConvolutionIdentifier::Center)),
        }?;

        Ok(ConvolutionIdx(new_coords, conv_id))
    }

    /// Positive k is left, counter clockwise
    /// Negative k is right, clockwise
    pub fn get_left_right_idx_from_center(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
        rk: isize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        // In the left right direction, unlike up down, every chunk has the same number of radial lines
        let radial_lines = target_chunk.get_chunk_coords().get_num_radial_lines();

        // You should not be doing any loops that might make you re-target yourself
        if rk.abs() >= radial_lines as isize {
            return Err(ConvOutOfBoundsError(ConvolutionIdx(
                JkVector {
                    j: pos.j,
                    k: modulo(pos.k as isize + rk, radial_lines),
                },
                ConvolutionIdentifier::Center,
            )));
        }

        let new_k = modulo(pos.k as isize + rk, radial_lines);
        match self.chunk_idxs.left_right {
            LeftRightNeighborIdxs::SingleChunkLayer => {
                let new_coords = JkVector { j: pos.j, k: new_k };
                Ok(ConvolutionIdx(new_coords, ConvolutionIdentifier::Center))
            }
            LeftRightNeighborIdxs::LR { .. } => {
                if pos.k as isize + rk >= radial_lines as isize {
                    Ok(ConvolutionIdx(
                        JkVector { j: pos.j, k: new_k },
                        ConvolutionIdentifier::LeftRight(LeftRightNeighborIdentifier::LR(
                            LeftRightNeighborIdentifierLR::Left,
                        )),
                    ))
                } else if pos.k as isize + rk < 0 {
                    Ok(ConvolutionIdx(
                        JkVector { j: pos.j, k: new_k },
                        ConvolutionIdentifier::LeftRight(LeftRightNeighborIdentifier::LR(
                            LeftRightNeighborIdentifierLR::Right,
                        )),
                    ))
                } else {
                    Ok(ConvolutionIdx(
                        JkVector { j: pos.j, k: new_k },
                        ConvolutionIdentifier::Center,
                    ))
                }
            }
        }
    }

    pub fn get_left_right_idx_from_bottom(
        &self,
        pos: &JkVector,
        chunk_id: BottomNeighborIdentifier,
        rk: isize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        unimplemented!()
    }
}

/// Getter and setter methods
impl ElementGridConvolutionNeighbors {
    pub fn get(
        &self,
        target_grid: &ElementGrid,
        idx: ConvolutionIdx,
    ) -> Result<&Box<dyn Element>, ConvOutOfBoundsError> {
        match idx.1 {
            ConvolutionIdentifier::Center => match target_grid.checked_get(idx.0) {
                Ok(element) => Ok(element),
                Err(_) => Err(ConvOutOfBoundsError(idx)),
            },
            ConvolutionIdentifier::Top(top_id) => self.grids.top.get(idx.0, top_id),
            ConvolutionIdentifier::Bottom(bottom_id) => self.grids.bottom.get(idx.0, bottom_id),
            ConvolutionIdentifier::LeftRight(left_right_id) => {
                self.grids.left_right.get(idx.0, left_right_id)
            }
        }
    }

    pub fn replace(
        &mut self,
        target_grid: &mut ElementGrid,
        idx: ConvolutionIdx,
        element: Box<dyn Element>,
        current_time: Clock,
    ) -> Result<Box<dyn Element>, ConvOutOfBoundsError> {
        match idx.1 {
            ConvolutionIdentifier::Center => {
                match target_grid.replace(idx.0, element, current_time) {
                    Ok(ele) => Ok(ele),
                    Err(_) => Err(ConvOutOfBoundsError(idx)),
                }
            }
            ConvolutionIdentifier::Top(top_id) => {
                self.grids.top.replace(idx.0, top_id, element, current_time)
            }
            ConvolutionIdentifier::Bottom(bottom_id) => {
                self.grids
                    .bottom
                    .replace(idx.0, bottom_id, element, current_time)
            }
            ConvolutionIdentifier::LeftRight(left_right_id) => {
                self.grids
                    .left_right
                    .replace(idx.0, left_right_id, element, current_time)
            }
        }
    }
}
