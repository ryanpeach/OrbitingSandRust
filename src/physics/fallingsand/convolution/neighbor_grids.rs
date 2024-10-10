//! Actual data storage types for the convolution
use hashbrown::HashMap;
use thiserror::Error;

use crate::physics::fallingsand::{
    data::element_grid::ElementGrid,
    elements::element::Element,
    util::vectors::{ChunkIjkVector, JkVector},
};

use super::{
    neighbor_identifiers::{
        BottomNeighborIdentifier, BottomNeighborIdentifierChunkDoubling,
        BottomNeighborIdentifierNormal, ConvolutionIdentifier, ConvolutionIdx,
        LeftRightNeighborIdentifier, TopNeighborIdentifier, TopNeighborIdentifierChunkDoubling,
        TopNeighborIdentifierNormal,
    },
    neighbor_indexes::{BottomNeighborIdxs, LeftRightNeighborIdxs, TopNeighborIdxs},
};

/// The main type exported by this module
/// Contains all the neighbor grids for the convolution
/// Check out the [`super::neighbor_identifiers::ConvolutionIdentifier`] and
///  [`super::neighbor_indexes::ElementGridConvolutionNeighborIdxs`] documentation for more information
pub struct ElementGridConvolutionNeighborGrids {
    /// The top neighbor grids
    pub top: TopNeighborGrids,
    /// The left and right neighbor grids
    pub left_right: LeftRightNeighborGrids,
    /// The bottom neighbor grids
    pub bottom: BottomNeighborGrids,
}

impl ElementGridConvolutionNeighborGrids {
    /// Converts the `ElementGridConvolutionNeighborGrids` into a hashmap
    #[must_use]
    pub fn into_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        let mut map = HashMap::new();
        map.extend(self.top.to_hashmap());
        map.extend(self.left_right.to_hashmap());
        map.extend(self.bottom.to_hashmap());
        map
    }
}

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone, Copy, Error)]
#[error("{:?} went outside the constraints of chunk {:?} and there are no further chunks", .0.0, .0.1)]
pub struct ConvOutOfBoundsError(pub ConvolutionIdx);

/// Left and Right neighbor grids in the convolution
/// Check out the  [`super::neighbor_identifiers::LeftRightNeighborIdentifier`] and
///  [`super::neighbor_indexes::LeftRightNeighborIdxs`] documentation for more information
/// documentation for more information
pub enum LeftRightNeighborGrids {
    /// The left and right elements
    /// TODO: Unecessary to have a struct for this, flatten into the enum
    LR {
        /// The left element
        l: ElementGrid,
        /// The right element
        r: ElementGrid,
    },
}

impl LeftRightNeighborGrids {
    /// Converts the `LeftRightNeighborGrids` into a hashmap
    #[must_use]
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            LeftRightNeighborGrids::LR { l, r } => {
                let mut map = HashMap::new();
                map.insert(l.coords().chunk_idx(), l);
                map.insert(r.coords().chunk_idx(), r);
                map
            }
        }
    }

    /// Converts a hashmap into a `LeftRightNeighborGrids`
    #[allow(clippy::unwrap_used)]
    pub fn from_hashmap(
        idxs: &LeftRightNeighborIdxs,
        grids: &mut HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        match idxs {
            LeftRightNeighborIdxs::LR { l, r } => LeftRightNeighborGrids::LR {
                l: grids.remove(l).unwrap(),
                r: grids.remove(r).unwrap(),
            },
        }
    }

    /// Gets the chunk at the given chunk index
    #[must_use]
    pub fn chunk_at_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, LeftRightNeighborIdentifier)> {
        match self {
            LeftRightNeighborGrids::LR { l, r } => {
                if l.coords().chunk_idx() == idx {
                    Some((l, LeftRightNeighborIdentifier::Left))
                } else if r.coords().chunk_idx() == idx {
                    Some((r, LeftRightNeighborIdentifier::Right))
                } else {
                    None
                }
            }
        }
    }
}

/// Top neighbor grids in the convolution
/// Check out the  [`super::neighbor_identifiers::TopNeighborIdentifier`] and
///  [`super::neighbor_indexes::TopNeighborIdxs`] documentation for more information
pub enum TopNeighborGrids {
    /// Indicates that there are the same number of chunks above as you have
    /// However, the cells may still double tangentially
    /// However, some enums need that information and others dont, so we
    /// dont want to add too much complexity to the match statements if we can help it
    Normal {
        /// Top left element
        tl: ElementGrid,
        /// Top element
        t: ElementGrid,
        /// Top right element
        tr: ElementGrid,
    },
    /// Indicates a **chunk doubling** layer transition
    ChunkDoubling {
        /// Top left element
        tl: ElementGrid,
        /// Second top center element, left of center
        t1: ElementGrid,
        /// First top center element, right of center
        t0: ElementGrid,
        /// Top right element
        tr: ElementGrid,
    },
    /// No more chunks above you
    TopOfGrid,
}

impl TopNeighborGrids {
    /// Converts the `TopNeighborGrids` into a hashmap
    #[must_use]
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            TopNeighborGrids::Normal { tl, t, tr } => {
                let mut map = HashMap::new();
                map.insert(tl.coords().chunk_idx(), tl);
                map.insert(t.coords().chunk_idx(), t);
                map.insert(tr.coords().chunk_idx(), tr);
                map
            }
            TopNeighborGrids::ChunkDoubling { tl, t1, t0, tr } => {
                let mut map = HashMap::new();
                map.insert(tl.coords().chunk_idx(), tl);
                map.insert(t1.coords().chunk_idx(), t1);
                map.insert(t0.coords().chunk_idx(), t0);
                map.insert(tr.coords().chunk_idx(), tr);
                map
            }
            TopNeighborGrids::TopOfGrid => HashMap::new(),
        }
    }

    /// Converts a hashmap into a `TopNeighborGrids`
    #[allow(clippy::unwrap_used)]
    pub fn from_hashmap(
        idxs: &TopNeighborIdxs,
        grids: &mut HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        match idxs {
            TopNeighborIdxs::Normal { tl, t, tr } => TopNeighborGrids::Normal {
                tl: grids.remove(tl).unwrap(),
                t: grids.remove(t).unwrap(),
                tr: grids.remove(tr).unwrap(),
            },
            TopNeighborIdxs::ChunkDoubling { tl, t1, t0, tr } => TopNeighborGrids::ChunkDoubling {
                tl: grids.remove(tl).unwrap(),
                t1: grids.remove(t1).unwrap(),
                t0: grids.remove(t0).unwrap(),
                tr: grids.remove(tr).unwrap(),
            },
            TopNeighborIdxs::TopOfGrid => TopNeighborGrids::TopOfGrid,
        }
    }

    /// Gets the element at the given index
    /// Returns an error if the given index is not in the convolution
    pub fn get(
        &self,
        idx: JkVector,
        top_neighbor_id: TopNeighborIdentifier,
    ) -> Result<&dyn Element, ConvOutOfBoundsError> {
        match top_neighbor_id {
            TopNeighborIdentifier::Normal(normal_id) => match normal_id {
                TopNeighborIdentifierNormal::Top => {
                    if let TopNeighborGrids::Normal { tl: _, t, tr: _ } = &self {
                        match t.checked_get(idx) {
                            Ok(element) => Ok(element),
                            Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                idx,
                                ConvolutionIdentifier::Top(top_neighbor_id),
                            ))),
                        }
                    } else {
                        panic!("The identifier said the index was from a normal top neighbor, but the top neighbor grids were not normal")
                    }
                }
                TopNeighborIdentifierNormal::TopLeft => {
                    if let TopNeighborGrids::Normal { tl, t: _, tr: _ } = &self {
                        match tl.checked_get(idx) {
                            Ok(element) => Ok(element),
                            Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                idx,
                                ConvolutionIdentifier::Top(top_neighbor_id),
                            ))),
                        }
                    } else {
                        panic!("The identifier said the index was from a normal top left neighbor, but the top neighbor grids were not normal")
                    }
                }
                TopNeighborIdentifierNormal::TopRight => {
                    if let TopNeighborGrids::Normal { tl: _, t: _, tr } = &self {
                        match tr.checked_get(idx) {
                            Ok(element) => Ok(element),
                            Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                idx,
                                ConvolutionIdentifier::Top(top_neighbor_id),
                            ))),
                        }
                    } else {
                        panic!("The identifier said the index was from a normal top right neighbor, but the top neighbor grids were not normal")
                    }
                }
            },
            TopNeighborIdentifier::ChunkDoubling(layer_transition_id) => {
                match layer_transition_id {
                    TopNeighborIdentifierChunkDoubling::Top0 => {
                        if let TopNeighborGrids::ChunkDoubling {
                            tl: _,
                            t0,
                            t1: _,
                            tr: _,
                        } = &self
                        {
                            match t0.checked_get(idx) {
                                Ok(element) => Ok(element),
                                Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                    idx,
                                    ConvolutionIdentifier::Top(top_neighbor_id),
                                ))),
                            }
                        } else {
                            panic!("The identifier said the index was from a layer transition top0 neighbor, but the top neighbor grids were not layer transition")
                        }
                    }
                    TopNeighborIdentifierChunkDoubling::Top1 => {
                        if let TopNeighborGrids::ChunkDoubling {
                            tl: _,
                            t0: _,
                            t1,
                            tr: _,
                        } = &self
                        {
                            match t1.checked_get(idx) {
                                Ok(element) => Ok(element),
                                Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                    idx,
                                    ConvolutionIdentifier::Top(top_neighbor_id),
                                ))),
                            }
                        } else {
                            panic!("The identifier said the index was from a layer transition top1 neighbor, but the top neighbor grids were not layer transition")
                        }
                    }
                    TopNeighborIdentifierChunkDoubling::TopLeft => {
                        if let TopNeighborGrids::ChunkDoubling {
                            tl,
                            t0: _,
                            t1: _,
                            tr: _,
                        } = &self
                        {
                            match tl.checked_get(idx) {
                                Ok(element) => Ok(element),
                                Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                    idx,
                                    ConvolutionIdentifier::Top(top_neighbor_id),
                                ))),
                            }
                        } else {
                            panic!("The identifier said the index was from a layer transition top left neighbor, but the top neighbor grids were not layer transition")
                        }
                    }
                    TopNeighborIdentifierChunkDoubling::TopRight => {
                        if let TopNeighborGrids::ChunkDoubling {
                            tl: _,
                            t0: _,
                            t1: _,
                            tr,
                        } = &self
                        {
                            match tr.checked_get(idx) {
                                Ok(element) => Ok(element),
                                Err(_) => Err(ConvOutOfBoundsError(ConvolutionIdx(
                                    idx,
                                    ConvolutionIdentifier::Top(top_neighbor_id),
                                ))),
                            }
                        } else {
                            panic!("The identifier said the index was from a layer transition top right neighbor, but the top neighbor grids were not layer transition")
                        }
                    }
                }
            }
        }
    }

    /// Gets the chunk at the given chunk index
    #[must_use]
    pub fn chunk_at_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, TopNeighborIdentifier)> {
        match self {
            TopNeighborGrids::Normal { tl, t, tr } => {
                if tl.coords().chunk_idx() == idx {
                    Some((
                        tl,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::TopLeft),
                    ))
                } else if t.coords().chunk_idx() == idx {
                    Some((
                        t,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::Top),
                    ))
                } else if tr.coords().chunk_idx() == idx {
                    Some((
                        tr,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::TopRight),
                    ))
                } else {
                    None
                }
            }
            TopNeighborGrids::ChunkDoubling { tl, t1, t0, tr } => {
                if tl.coords().chunk_idx() == idx {
                    Some((
                        tl,
                        TopNeighborIdentifier::ChunkDoubling(
                            TopNeighborIdentifierChunkDoubling::TopLeft,
                        ),
                    ))
                } else if t1.coords().chunk_idx() == idx {
                    Some((
                        t1,
                        TopNeighborIdentifier::ChunkDoubling(
                            TopNeighborIdentifierChunkDoubling::Top1,
                        ),
                    ))
                } else if t0.coords().chunk_idx() == idx {
                    Some((
                        t0,
                        TopNeighborIdentifier::ChunkDoubling(
                            TopNeighborIdentifierChunkDoubling::Top0,
                        ),
                    ))
                } else if tr.coords().chunk_idx() == idx {
                    Some((
                        tr,
                        TopNeighborIdentifier::ChunkDoubling(
                            TopNeighborIdentifierChunkDoubling::TopRight,
                        ),
                    ))
                } else {
                    None
                }
            }
            TopNeighborGrids::TopOfGrid => None,
        }
    }

    /// Gets the number of concentric circles in the top layer of the convolution
    #[must_use]
    pub fn num_concentric_circles(&self) -> u32 {
        match self {
            TopNeighborGrids::Normal { tl: _, t, tr: _ } => t.coords().num_concentric_circles(),
            TopNeighborGrids::ChunkDoubling {
                tl,
                t1: _,
                t0: _,
                tr: _,
            } => tl.coords().num_concentric_circles(),
            TopNeighborGrids::TopOfGrid => 0,
        }
    }

    /// Gets the number of radial lines in the top layer of the convolution
    #[must_use]
    pub fn num_radial_lines(&self) -> u32 {
        match self {
            TopNeighborGrids::Normal { tl: _, t, tr: _ } => t.coords().num_radial_lines(),
            TopNeighborGrids::ChunkDoubling {
                tl,
                t1: _,
                t0: _,
                tr: _,
            } => tl.coords().num_radial_lines(),
            TopNeighborGrids::TopOfGrid => 0,
        }
    }
}

/// Bottom neighbor grids in the convolution
/// Check out the  [`super::neighbor_identifiers::BottomNeighborIdentifier`] and
///  [`super::neighbor_indexes::BottomNeighborIdxs`] documentation for more information
pub enum BottomNeighborGrids {
    /// Indicates that there are the same number of chunks below as you have
    /// However, the cells may still half tangentially
    /// However, some enums need that information and others dont, so we
    /// dont want to add too much complexity to the match statements if we can help it
    Normal {
        /// Bottom left element
        bl: ElementGrid,
        /// Bottom element
        b: ElementGrid,
        /// Bottom right element
        br: ElementGrid,
    },
    /// Indicates a **chunk doubling** layer transition
    /// One of these will be directly below you, and be bigger than you off to one direction
    /// Whereas the other will be diagonally below you
    /// This depends on if your  [`ChunkIjkVector`] has a `k` value which is even or odd
    /// If it is even, then the `bl` will be directly below you, and you will be straddling its right side
    /// If it is odd, then the `br` will be directly below you, and you will be straddling its left side
    ChunkDoubling {
        /// Bottom left element
        bl: ElementGrid,
        /// Bottom right element
        br: ElementGrid,
    },
    /// No more chunks below you
    BottomOfGrid,
}

impl BottomNeighborGrids {
    /// Converts the `BottomNeighborGrids` into a hashmap
    #[must_use]
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            BottomNeighborGrids::Normal { bl, b, br } => {
                let mut map = HashMap::new();
                map.insert(bl.coords().chunk_idx(), bl);
                map.insert(b.coords().chunk_idx(), b);
                map.insert(br.coords().chunk_idx(), br);
                map
            }
            BottomNeighborGrids::ChunkDoubling { bl, br } => {
                let mut map = HashMap::new();
                map.insert(bl.coords().chunk_idx(), bl);
                map.insert(br.coords().chunk_idx(), br);
                map
            }
            BottomNeighborGrids::BottomOfGrid => HashMap::new(),
        }
    }

    /// Converts a hashmap into a `BottomNeighborGrids`
    #[allow(clippy::unwrap_used)]
    pub fn from_hashmap(
        idxs: &BottomNeighborIdxs,
        grids: &mut HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        match idxs {
            BottomNeighborIdxs::Normal { bl, b, br } => BottomNeighborGrids::Normal {
                bl: grids.remove(bl).unwrap(),
                b: grids.remove(b).unwrap(),
                br: grids.remove(br).unwrap(),
            },
            BottomNeighborIdxs::ChunkDoubling { bl, br } => BottomNeighborGrids::ChunkDoubling {
                bl: grids.remove(bl).unwrap(),
                br: grids.remove(br).unwrap(),
            },
            BottomNeighborIdxs::BottomOfGrid => BottomNeighborGrids::BottomOfGrid,
        }
    }

    /// Gets the element at the given index
    #[must_use]
    pub fn chunk_at_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, BottomNeighborIdentifier)> {
        match self {
            BottomNeighborGrids::Normal { bl, b, br } => {
                if bl.coords().chunk_idx() == idx {
                    Some((
                        bl,
                        BottomNeighborIdentifier::Normal(
                            BottomNeighborIdentifierNormal::BottomLeft,
                        ),
                    ))
                } else if b.coords().chunk_idx() == idx {
                    Some((
                        b,
                        BottomNeighborIdentifier::Normal(BottomNeighborIdentifierNormal::Bottom),
                    ))
                } else if br.coords().chunk_idx() == idx {
                    Some((
                        br,
                        BottomNeighborIdentifier::Normal(
                            BottomNeighborIdentifierNormal::BottomRight,
                        ),
                    ))
                } else {
                    None
                }
            }
            BottomNeighborGrids::ChunkDoubling { bl, br } => {
                if bl.coords().chunk_idx() == idx {
                    Some((
                        bl,
                        BottomNeighborIdentifier::ChunkDoubling(
                            BottomNeighborIdentifierChunkDoubling::BottomLeft,
                        ),
                    ))
                } else if br.coords().chunk_idx() == idx {
                    Some((
                        br,
                        BottomNeighborIdentifier::ChunkDoubling(
                            BottomNeighborIdentifierChunkDoubling::BottomRight,
                        ),
                    ))
                } else {
                    None
                }
            }
            BottomNeighborGrids::BottomOfGrid => None,
        }
    }

    /// Gets the number of radial lines in the bottom layer of the convolution
    #[must_use]
    pub fn num_radial_lines(&self) -> u32 {
        match self {
            BottomNeighborGrids::Normal { bl: _, b, br: _ } => b.coords().num_radial_lines(),
            BottomNeighborGrids::ChunkDoubling { bl, br: _ } => bl.coords().num_radial_lines(),
            BottomNeighborGrids::BottomOfGrid => 0,
        }
    }

    /// Gets the number of concentric circles in the bottom layer of the convolution
    #[must_use]
    pub fn num_concentric_circles(&self) -> u32 {
        match self {
            BottomNeighborGrids::Normal { bl: _, b, br: _ } => b.coords().num_concentric_circles(),
            BottomNeighborGrids::ChunkDoubling { bl, br: _ } => {
                bl.coords().num_concentric_circles()
            }
            BottomNeighborGrids::BottomOfGrid => 0,
        }
    }
}
