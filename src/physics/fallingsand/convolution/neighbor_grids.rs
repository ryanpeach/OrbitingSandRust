use std::fmt;

use hashbrown::HashMap;

use crate::physics::fallingsand::{
    data::element_grid::ElementGrid,
    elements::element::Element,
    util::vectors::{ChunkIjkVector, JkVector},
};

use super::{neighbor_identifiers::*, neighbor_indexes::*};

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone, Copy)]
pub struct ConvOutOfBoundsError(pub ConvolutionIdx);
impl fmt::Display for ConvOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} went outside the constraints of chunk {:?} and there are no further chunks",
            self.0 .0, self.0 .1
        )
    }
}

#[allow(clippy::large_enum_variant)]
pub enum LeftRightNeighborGrids {
    LR { l: ElementGrid, r: ElementGrid },
    SingleChunkLayer,
}

impl LeftRightNeighborGrids {
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            LeftRightNeighborGrids::LR { l, r } => {
                let mut map = HashMap::new();
                map.insert(l.get_chunk_coords().get_chunk_idx(), l);
                map.insert(r.get_chunk_coords().get_chunk_idx(), r);
                map
            }
            LeftRightNeighborGrids::SingleChunkLayer => HashMap::new(),
        }
    }

    pub fn from_hashmap(
        idxs: &LeftRightNeighborIdxs,
        grids: &mut HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        match idxs {
            LeftRightNeighborIdxs::LR { l, r } => LeftRightNeighborGrids::LR {
                l: grids.remove(l).unwrap(),
                r: grids.remove(r).unwrap(),
            },
            LeftRightNeighborIdxs::SingleChunkLayer => LeftRightNeighborGrids::SingleChunkLayer,
        }
    }

    pub fn get_chunk_by_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, LeftRightNeighborIdentifier)> {
        match self {
            LeftRightNeighborGrids::LR { l, r } => {
                if l.get_chunk_coords().get_chunk_idx() == idx {
                    Some((l, LeftRightNeighborIdentifier::Left))
                } else if r.get_chunk_coords().get_chunk_idx() == idx {
                    Some((r, LeftRightNeighborIdentifier::Right))
                } else {
                    None
                }
            }
            LeftRightNeighborGrids::SingleChunkLayer => None,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum TopNeighborGrids {
    Normal {
        tl: ElementGrid,
        t: ElementGrid,
        tr: ElementGrid,
    },
    LayerTransition {
        tl: ElementGrid,
        t1: ElementGrid,
        t0: ElementGrid,
        tr: ElementGrid,
    },
    TopOfGrid,
}

impl TopNeighborGrids {
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            TopNeighborGrids::Normal { tl, t, tr } => {
                let mut map = HashMap::new();
                map.insert(tl.get_chunk_coords().get_chunk_idx(), tl);
                map.insert(t.get_chunk_coords().get_chunk_idx(), t);
                map.insert(tr.get_chunk_coords().get_chunk_idx(), tr);
                map
            }
            TopNeighborGrids::LayerTransition { tl, t1, t0, tr } => {
                let mut map = HashMap::new();
                map.insert(tl.get_chunk_coords().get_chunk_idx(), tl);
                map.insert(t1.get_chunk_coords().get_chunk_idx(), t1);
                map.insert(t0.get_chunk_coords().get_chunk_idx(), t0);
                map.insert(tr.get_chunk_coords().get_chunk_idx(), tr);
                map
            }
            TopNeighborGrids::TopOfGrid => HashMap::new(),
        }
    }

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
            TopNeighborIdxs::LayerTransition { tl, t1, t0, tr } => {
                TopNeighborGrids::LayerTransition {
                    tl: grids.remove(tl).unwrap(),
                    t1: grids.remove(t1).unwrap(),
                    t0: grids.remove(t0).unwrap(),
                    tr: grids.remove(tr).unwrap(),
                }
            }
            TopNeighborIdxs::TopOfGrid => TopNeighborGrids::TopOfGrid,
        }
    }

    #[allow(clippy::borrowed_box)]
    pub fn get(
        &self,
        idx: JkVector,
        top_neighbor_id: TopNeighborIdentifier,
    ) -> Result<&Box<dyn Element>, ConvOutOfBoundsError> {
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
            TopNeighborIdentifier::LayerTransition(layer_transition_id) => {
                match layer_transition_id {
                    TopNeighborIdentifierLayerTransition::Top0 => {
                        if let TopNeighborGrids::LayerTransition {
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
                    TopNeighborIdentifierLayerTransition::Top1 => {
                        if let TopNeighborGrids::LayerTransition {
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
                    TopNeighborIdentifierLayerTransition::TopLeft => {
                        if let TopNeighborGrids::LayerTransition {
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
                    TopNeighborIdentifierLayerTransition::TopRight => {
                        if let TopNeighborGrids::LayerTransition {
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

    pub fn get_chunk_by_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, TopNeighborIdentifier)> {
        match self {
            TopNeighborGrids::Normal { tl, t, tr } => {
                if tl.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        tl,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::TopLeft),
                    ))
                } else if t.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        t,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::Top),
                    ))
                } else if tr.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        tr,
                        TopNeighborIdentifier::Normal(TopNeighborIdentifierNormal::TopRight),
                    ))
                } else {
                    None
                }
            }
            TopNeighborGrids::LayerTransition { tl, t1, t0, tr } => {
                if tl.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        tl,
                        TopNeighborIdentifier::LayerTransition(
                            TopNeighborIdentifierLayerTransition::TopLeft,
                        ),
                    ))
                } else if t1.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        t1,
                        TopNeighborIdentifier::LayerTransition(
                            TopNeighborIdentifierLayerTransition::Top1,
                        ),
                    ))
                } else if t0.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        t0,
                        TopNeighborIdentifier::LayerTransition(
                            TopNeighborIdentifierLayerTransition::Top0,
                        ),
                    ))
                } else if tr.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        tr,
                        TopNeighborIdentifier::LayerTransition(
                            TopNeighborIdentifierLayerTransition::TopRight,
                        ),
                    ))
                } else {
                    None
                }
            }
            TopNeighborGrids::TopOfGrid => None,
        }
    }

    pub fn get_num_concentric_circles(&self) -> usize {
        match self {
            TopNeighborGrids::Normal { tl: _, t, tr: _ } => {
                t.get_chunk_coords().get_num_concentric_circles()
            }
            TopNeighborGrids::LayerTransition {
                tl,
                t1: _,
                t0: _,
                tr: _,
            } => tl.get_chunk_coords().get_num_concentric_circles(),
            TopNeighborGrids::TopOfGrid => 0,
        }
    }

    pub fn get_num_radial_lines(&self) -> usize {
        match self {
            TopNeighborGrids::Normal { tl: _, t, tr: _ } => {
                t.get_chunk_coords().get_num_radial_lines()
            }
            TopNeighborGrids::LayerTransition {
                tl,
                t1: _,
                t0: _,
                tr: _,
            } => tl.get_chunk_coords().get_num_radial_lines(),
            TopNeighborGrids::TopOfGrid => 0,
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum BottomNeighborGrids {
    Normal {
        bl: ElementGrid,
        b: ElementGrid,
        br: ElementGrid,
    },
    LayerTransition {
        bl: ElementGrid,
        br: ElementGrid,
    },
    BottomOfGrid,
}

impl BottomNeighborGrids {
    pub fn to_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        match self {
            BottomNeighborGrids::Normal { bl, b, br } => {
                let mut map = HashMap::new();
                map.insert(bl.get_chunk_coords().get_chunk_idx(), bl);
                map.insert(b.get_chunk_coords().get_chunk_idx(), b);
                map.insert(br.get_chunk_coords().get_chunk_idx(), br);
                map
            }
            BottomNeighborGrids::LayerTransition { bl, br } => {
                let mut map = HashMap::new();
                map.insert(bl.get_chunk_coords().get_chunk_idx(), bl);
                map.insert(br.get_chunk_coords().get_chunk_idx(), br);
                map
            }
            BottomNeighborGrids::BottomOfGrid => HashMap::new(),
        }
    }

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
            BottomNeighborIdxs::LayerTransition { bl, br } => {
                BottomNeighborGrids::LayerTransition {
                    bl: grids.remove(bl).unwrap(),
                    br: grids.remove(br).unwrap(),
                }
            }
            BottomNeighborIdxs::BottomOfGrid => BottomNeighborGrids::BottomOfGrid,
        }
    }

    pub fn get_chunk_by_chunk_ijk(
        &self,
        idx: ChunkIjkVector,
    ) -> Option<(&ElementGrid, BottomNeighborIdentifier)> {
        match self {
            BottomNeighborGrids::Normal { bl, b, br } => {
                if bl.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        bl,
                        BottomNeighborIdentifier::Normal(
                            BottomNeighborIdentifierNormal::BottomLeft,
                        ),
                    ))
                } else if b.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        b,
                        BottomNeighborIdentifier::Normal(BottomNeighborIdentifierNormal::Bottom),
                    ))
                } else if br.get_chunk_coords().get_chunk_idx() == idx {
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
            BottomNeighborGrids::LayerTransition { bl, br } => {
                if bl.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        bl,
                        BottomNeighborIdentifier::LayerTransition(
                            BottomNeighborIdentifierLayerTransition::BottomLeft,
                        ),
                    ))
                } else if br.get_chunk_coords().get_chunk_idx() == idx {
                    Some((
                        br,
                        BottomNeighborIdentifier::LayerTransition(
                            BottomNeighborIdentifierLayerTransition::BottomRight,
                        ),
                    ))
                } else {
                    None
                }
            }
            BottomNeighborGrids::BottomOfGrid => None,
        }
    }

    pub fn get_num_radial_lines(&self) -> usize {
        match self {
            BottomNeighborGrids::Normal { bl: _, b, br: _ } => {
                b.get_chunk_coords().get_num_radial_lines()
            }
            BottomNeighborGrids::LayerTransition { bl, br: _ } => {
                bl.get_chunk_coords().get_num_radial_lines()
            }
            BottomNeighborGrids::BottomOfGrid => 0,
        }
    }

    pub fn get_num_concentric_circles(&self) -> usize {
        match self {
            BottomNeighborGrids::Normal { bl: _, b, br: _ } => {
                b.get_chunk_coords().get_num_concentric_circles()
            }
            BottomNeighborGrids::LayerTransition { bl, br: _ } => {
                bl.get_chunk_coords().get_num_concentric_circles()
            }
            BottomNeighborGrids::BottomOfGrid => 0,
        }
    }
}

pub struct ElementGridConvolutionNeighborGrids {
    pub top: TopNeighborGrids,
    pub left_right: LeftRightNeighborGrids,
    pub bottom: BottomNeighborGrids,
}

impl ElementGridConvolutionNeighborGrids {
    pub fn into_hashmap(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        let mut map = HashMap::new();
        map.extend(self.top.to_hashmap());
        map.extend(self.left_right.to_hashmap());
        map.extend(self.bottom.to_hashmap());
        map
    }
}
