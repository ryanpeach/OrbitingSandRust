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
                JkVector { j: pos.j, k: pos.k },
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
        _pos: &JkVector,
        _chunk_id: BottomNeighborIdentifier,
        _rk: isize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        unimplemented!()
    }
}

pub enum GetChunkErr {
    ReturnsVector,
}

/// Getter and setter methods
impl ElementGridConvolutionNeighbors {
    /// TODO, use macros to get rid of redundancy
    fn get_chunk_mut(
        &mut self,
        id: ConvolutionIdentifier,
    ) -> Result<&mut ElementGrid, GetChunkErr> {
        match id {
            ConvolutionIdentifier::Top(top_id) => match top_id {
                TopNeighborIdentifier::Normal(normal_id) => match normal_id {
                    TopNeighborIdentifierNormal::Top { .. } => {
                        if let TopNeighborGrids::Normal { t, .. } = &mut self.grids.top {
                            Ok(t)
                        } else {
                            panic!("Tried to get t chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopLeft { .. } => {
                        if let TopNeighborGrids::Normal { tl, .. } = &mut self.grids.top {
                            Ok(tl)
                        } else {
                            panic!("Tried to get tl chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopRight { .. } => {
                        if let TopNeighborGrids::Normal { tr, .. } = &mut self.grids.top {
                            Ok(tr)
                        } else {
                            panic!("Tried to get tr chunk that doesn't exist")
                        }
                    }
                },
                TopNeighborIdentifier::LayerTransition(layer_transition_id) => {
                    match layer_transition_id {
                        TopNeighborIdentifierLayerTransition::Top0 { .. } => {
                            if let TopNeighborGrids::LayerTransition { t0, .. } =
                                &mut self.grids.top
                            {
                                Ok(t0)
                            } else {
                                panic!("Tried to get t0 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::Top1 { .. } => {
                            if let TopNeighborGrids::LayerTransition { t1, .. } =
                                &mut self.grids.top
                            {
                                Ok(t1)
                            } else {
                                panic!("Tried to get t1 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::TopLeft { .. } => {
                            if let TopNeighborGrids::LayerTransition { tl, .. } =
                                &mut self.grids.top
                            {
                                Ok(tl)
                            } else {
                                panic!("Tried to get tl chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::TopRight { .. } => {
                            if let TopNeighborGrids::LayerTransition { tr, .. } =
                                &mut self.grids.top
                            {
                                Ok(tr)
                            } else {
                                panic!("Tried to get tr chunk that doesn't exist")
                            }
                        }
                    }
                }
                TopNeighborIdentifier::SingleChunkLayerAbove { .. } => {
                    if let TopNeighborGrids::SingleChunkLayerAbove { t, .. } = &mut self.grids.top {
                        Ok(t)
                    } else {
                        panic!("Tried to get t chunk that doesn't exist")
                    }
                }
                TopNeighborIdentifier::MultiChunkLayerAbove { .. } => {
                    if let TopNeighborGrids::MultiChunkLayerAbove { .. } = &mut self.grids.top {
                        Err(GetChunkErr::ReturnsVector)
                    } else {
                        panic!("Tried to get t chunk that doesn't exist")
                    }
                }
            },
            ConvolutionIdentifier::Bottom(bottom_id) => match bottom_id {
                BottomNeighborIdentifier::Normal(normal_id) => match normal_id {
                    BottomNeighborIdentifierNormal::Bottom { .. } => {
                        if let BottomNeighborGrids::Normal { b, .. } = &mut self.grids.bottom {
                            Ok(b)
                        } else {
                            panic!("Tried to get b chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomLeft { .. } => {
                        if let BottomNeighborGrids::Normal { bl, .. } = &mut self.grids.bottom {
                            Ok(bl)
                        } else {
                            panic!("Tried to get bl chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomRight { .. } => {
                        if let BottomNeighborGrids::Normal { br, .. } = &mut self.grids.bottom {
                            Ok(br)
                        } else {
                            panic!("Tried to get br chunk that doesn't exist")
                        }
                    }
                },
                BottomNeighborIdentifier::LayerTransition(layer_transition_id) => {
                    match layer_transition_id {
                        BottomNeighborIdentifierLayerTransition::BottomLeft { .. } => {
                            if let BottomNeighborGrids::LayerTransition { bl, .. } =
                                &mut self.grids.bottom
                            {
                                Ok(bl)
                            } else {
                                panic!("Tried to get bl chunk that doesn't exist")
                            }
                        }
                        BottomNeighborIdentifierLayerTransition::BottomRight { .. } => {
                            if let BottomNeighborGrids::LayerTransition { br, .. } =
                                &mut self.grids.bottom
                            {
                                Ok(br)
                            } else {
                                panic!("Tried to get br chunk that doesn't exist")
                            }
                        }
                    }
                }
                BottomNeighborIdentifier::FullLayerBelow { .. } => {
                    if let BottomNeighborGrids::FullLayerBelow { b, .. } = &mut self.grids.bottom {
                        Ok(b)
                    } else {
                        panic!("Tried to get b chunk that doesn't exist")
                    }
                }
            },
            ConvolutionIdentifier::LeftRight(left_right_id) => match left_right_id {
                LeftRightNeighborIdentifier::LR(lr_id) => match lr_id {
                    LeftRightNeighborIdentifierLR::Left { .. } => {
                        if let LeftRightNeighborGrids::LR { l, .. } = &mut self.grids.left_right {
                            Ok(l)
                        } else {
                            panic!("Tried to get l chunk that doesn't exist")
                        }
                    }
                    LeftRightNeighborIdentifierLR::Right { .. } => {
                        if let LeftRightNeighborGrids::LR { r, .. } = &mut self.grids.left_right {
                            Ok(r)
                        } else {
                            panic!("Tried to get r chunk that doesn't exist")
                        }
                    }
                },
            },
            ConvolutionIdentifier::Center => panic!("Tried to get the center chunk"),
        }
    }

    fn get_chunk(&self, id: ConvolutionIdentifier) -> Result<&ElementGrid, GetChunkErr> {
        match id {
            ConvolutionIdentifier::Top(top_id) => match top_id {
                TopNeighborIdentifier::Normal(normal_id) => match normal_id {
                    TopNeighborIdentifierNormal::Top { .. } => {
                        if let TopNeighborGrids::Normal { t, .. } = &self.grids.top {
                            Ok(t)
                        } else {
                            panic!("Tried to get t chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopLeft { .. } => {
                        if let TopNeighborGrids::Normal { tl, .. } = &self.grids.top {
                            Ok(tl)
                        } else {
                            panic!("Tried to get tl chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopRight { .. } => {
                        if let TopNeighborGrids::Normal { tr, .. } = &self.grids.top {
                            Ok(tr)
                        } else {
                            panic!("Tried to get tr chunk that doesn't exist")
                        }
                    }
                },
                TopNeighborIdentifier::LayerTransition(layer_transition_id) => {
                    match layer_transition_id {
                        TopNeighborIdentifierLayerTransition::Top0 { .. } => {
                            if let TopNeighborGrids::LayerTransition { t0, .. } = &self.grids.top {
                                Ok(t0)
                            } else {
                                panic!("Tried to get t0 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::Top1 { .. } => {
                            if let TopNeighborGrids::LayerTransition { t1, .. } = &self.grids.top {
                                Ok(t1)
                            } else {
                                panic!("Tried to get t1 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::TopLeft { .. } => {
                            if let TopNeighborGrids::LayerTransition { tl, .. } = &self.grids.top {
                                Ok(tl)
                            } else {
                                panic!("Tried to get tl chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierLayerTransition::TopRight { .. } => {
                            if let TopNeighborGrids::LayerTransition { tr, .. } = &self.grids.top {
                                Ok(tr)
                            } else {
                                panic!("Tried to get tr chunk that doesn't exist")
                            }
                        }
                    }
                }
                TopNeighborIdentifier::SingleChunkLayerAbove { .. } => {
                    if let TopNeighborGrids::SingleChunkLayerAbove { t, .. } = &self.grids.top {
                        Ok(t)
                    } else {
                        panic!("Tried to get t chunk that doesn't exist")
                    }
                }
                TopNeighborIdentifier::MultiChunkLayerAbove { .. } => {
                    if let TopNeighborGrids::MultiChunkLayerAbove { .. } = &self.grids.top {
                        Err(GetChunkErr::ReturnsVector)
                    } else {
                        panic!("Tried to get t chunk that doesn't exist")
                    }
                }
            },
            ConvolutionIdentifier::Bottom(bottom_id) => match bottom_id {
                BottomNeighborIdentifier::Normal(normal_id) => match normal_id {
                    BottomNeighborIdentifierNormal::Bottom { .. } => {
                        if let BottomNeighborGrids::Normal { b, .. } = &self.grids.bottom {
                            Ok(b)
                        } else {
                            panic!("Tried to get b chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomLeft { .. } => {
                        if let BottomNeighborGrids::Normal { bl, .. } = &self.grids.bottom {
                            Ok(bl)
                        } else {
                            panic!("Tried to get bl chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomRight { .. } => {
                        if let BottomNeighborGrids::Normal { br, .. } = &self.grids.bottom {
                            Ok(br)
                        } else {
                            panic!("Tried to get br chunk that doesn't exist")
                        }
                    }
                },
                BottomNeighborIdentifier::LayerTransition(layer_transition_id) => {
                    match layer_transition_id {
                        BottomNeighborIdentifierLayerTransition::BottomLeft { .. } => {
                            if let BottomNeighborGrids::LayerTransition { bl, .. } =
                                &self.grids.bottom
                            {
                                Ok(bl)
                            } else {
                                panic!("Tried to get bl chunk that doesn't exist")
                            }
                        }
                        BottomNeighborIdentifierLayerTransition::BottomRight { .. } => {
                            if let BottomNeighborGrids::LayerTransition { br, .. } =
                                &self.grids.bottom
                            {
                                Ok(br)
                            } else {
                                panic!("Tried to get br chunk that doesn't exist")
                            }
                        }
                    }
                }
                BottomNeighborIdentifier::FullLayerBelow { .. } => {
                    if let BottomNeighborGrids::FullLayerBelow { b, .. } = &self.grids.bottom {
                        Ok(b)
                    } else {
                        panic!("Tried to get b chunk that doesn't exist")
                    }
                }
            },
            ConvolutionIdentifier::LeftRight(left_right_id) => match left_right_id {
                LeftRightNeighborIdentifier::LR(lr_id) => match lr_id {
                    LeftRightNeighborIdentifierLR::Left { .. } => {
                        if let LeftRightNeighborGrids::LR { l, .. } = &self.grids.left_right {
                            Ok(l)
                        } else {
                            panic!("Tried to get l chunk that doesn't exist")
                        }
                    }
                    LeftRightNeighborIdentifierLR::Right { .. } => {
                        if let LeftRightNeighborGrids::LR { r, .. } = &self.grids.left_right {
                            Ok(r)
                        } else {
                            panic!("Tried to get r chunk that doesn't exist")
                        }
                    }
                },
            },
            ConvolutionIdentifier::Center => panic!("Tried to get the center chunk"),
        }
    }

    pub fn get(
        &self,
        target_grid: &ElementGrid,
        idx: ConvolutionIdx,
    ) -> Result<Box<dyn Element>, ConvOutOfBoundsError> {
        match idx.1 {
            ConvolutionIdentifier::Center => match target_grid.checked_get(idx.0) {
                Ok(element) => Ok(element.box_clone()),
                Err(_) => Err(ConvOutOfBoundsError(idx)),
            },
            _ => match self.get_chunk(idx.1) {
                Ok(chunk) => match chunk.checked_get(idx.0) {
                    Ok(element) => Ok(element.box_clone()),
                    Err(_) => Err(ConvOutOfBoundsError(idx)),
                },
                Err(GetChunkErr::ReturnsVector) => {
                    unimplemented!("This is fairly easy to implement but not yet needed.")
                }
            },
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
                let out = target_grid.replace(idx.0, element, current_time);
                Ok(out)
            }
            _ => match self.get_chunk_mut(idx.1) {
                Ok(chunk) => {
                    let out = chunk.replace(idx.0, element, current_time);
                    Ok(out)
                }
                Err(GetChunkErr::ReturnsVector) => {
                    unimplemented!("This is fairly easy to implement but not yet needed.")
                }
            },
        }
    }
}
