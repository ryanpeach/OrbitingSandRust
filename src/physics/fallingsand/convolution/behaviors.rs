use std::fmt;

use hashbrown::HashMap;

use crate::physics::fallingsand::{
    element_grid::ElementGrid,
    util::{
        functions::modulo,
        vectors::{ChunkIjkVector, JkVector},
    },
};

use super::{
    neighbor_grids::ElementGridConvolutionNeighborGrids,
    neighbor_identifiers::{
        BottomNeighborIdentifier, BottomNeighborIdentifierLayerTransition,
        BottomNeighborIdentifierNormal, ConvolutionIdentifier, ConvolutionIdx,
        LeftRightNeighborIdentifier, LeftRightNeighborIdentifierLR,
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
        _chunk_idxs: ElementGridConvolutionNeighborIdxs,
        _target_idx: ChunkIjkVector,
        _grids: HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        unimplemented!()
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

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone)]
pub struct OutOfBoundsError(pub ConvolutionIdx);
impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} went outside the constraints of chunk {:?} and there are no further chunks",
            self.0 .0, self.0 .1
        )
    }
}

/// Behavior Methods
/// Methods which allow you to get an index "relative" to another index on the element convolution.
/// All of these methods are given "lr priority". Meaning if you ask for rel_idx RelJkVector(-1, -1) that means
/// go clockwise one then go down one. This is easier to program as lr is easy.
impl ElementGridConvolutionNeighbors {
    /// Gets the element n cells below the position in the target chunk
    /// the target chunk is the chunk in the center of the convolution
    /// the pos is the position in the target chunk
    pub fn get_below(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
        n: usize,
    ) -> Result<ConvolutionIdx, OutOfBoundsError> {
        let b_concentric_circles = self.grids.bottom.get_num_concentric_circles();
        let (new_j, new_k) = if pos.j >= n {
            (pos.j - n, pos.k)
        } else if pos.j as isize - n as isize + b_concentric_circles as isize >= 0 {
            (pos.j + b_concentric_circles - n, pos.k / 2)
        } else {
            return Err(OutOfBoundsError(ConvolutionIdx(
                JkVector {
                    j: pos.j - n,
                    k: pos.k,
                },
                ConvolutionIdentifier::Center,
            )));
        };

        let new_coords = JkVector { j: new_j, k: new_k };

        let conv_id = match self.chunk_idxs.bottom {
            BottomNeighborIdxs::BottomOfGrid if pos.j == 0 => Err(OutOfBoundsError(
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
    pub fn get_left_right(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
        rk: isize,
    ) -> Result<ConvolutionIdx, OutOfBoundsError> {
        // In the left right direction, unlike up down, every chunk has the same number of radial lines
        let radial_lines = target_chunk.get_chunk_coords().get_num_radial_lines();

        // You should not be doing any loops that might make you re-target yourself
        if rk.abs() >= radial_lines as isize {
            return Err(OutOfBoundsError(ConvolutionIdx(
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
}
