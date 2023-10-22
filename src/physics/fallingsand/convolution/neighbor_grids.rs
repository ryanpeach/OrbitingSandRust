use std::collections::HashMap;

use crate::physics::fallingsand::{element_grid::ElementGrid, util::vectors::ChunkIjkVector};

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
            LeftRightNeighborGrids::SingleChunkLayer => {
                let mut map = HashMap::new();
                map
            }
        }
    }
}

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
    SingleChunkLayerAbove {
        t: ElementGrid,
    },
    MultiChunkLayerAbove {
        chunks: Vec<ElementGrid>,
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
            TopNeighborGrids::SingleChunkLayerAbove { t } => {
                let mut map = HashMap::new();
                map.insert(t.get_chunk_coords().get_chunk_idx(), t);
                map
            }
            TopNeighborGrids::MultiChunkLayerAbove { chunks } => {
                let mut map = HashMap::new();
                for chunk in chunks {
                    map.insert(chunk.get_chunk_coords().get_chunk_idx(), chunk);
                }
                map
            }
            TopNeighborGrids::TopOfGrid => {
                let mut map = HashMap::new();
                map
            }
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
            TopNeighborGrids::SingleChunkLayerAbove { t } => {
                t.get_chunk_coords().get_num_concentric_circles()
            }
            TopNeighborGrids::MultiChunkLayerAbove { chunks } => {
                chunks[0].get_chunk_coords().get_num_concentric_circles()
            }
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
            TopNeighborGrids::SingleChunkLayerAbove { t } => {
                t.get_chunk_coords().get_num_radial_lines()
            }
            TopNeighborGrids::MultiChunkLayerAbove { chunks } => {
                chunks[0].get_chunk_coords().get_num_radial_lines()
            }
            TopNeighborGrids::TopOfGrid => 0,
        }
    }
}

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
    FullLayerBelow {
        b: ElementGrid,
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
            BottomNeighborGrids::FullLayerBelow { b } => {
                let mut map = HashMap::new();
                map.insert(b.get_chunk_coords().get_chunk_idx(), b);
                map
            }
            BottomNeighborGrids::BottomOfGrid => {
                let mut map = HashMap::new();
                map
            }
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
            BottomNeighborGrids::FullLayerBelow { b } => {
                b.get_chunk_coords().get_num_radial_lines()
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
            BottomNeighborGrids::FullLayerBelow { b } => {
                b.get_chunk_coords().get_num_concentric_circles()
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
