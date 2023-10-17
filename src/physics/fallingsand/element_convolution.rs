use std::collections::HashMap;

use super::{element_grid::ElementGrid, util::vectors::ChunkIjkVector};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeftRightNeighbors {
    LR {
        l: ChunkIjkVector,
        r: ChunkIjkVector,
    },
    SingleChunkLayer,
}

pub struct LeftRightNeighborsIter {
    lr: Option<LeftRightNeighbors>,
    index: usize,
}

impl Iterator for LeftRightNeighborsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lr {
            Some(LeftRightNeighbors::LR { l, r }) => {
                self.index += 1;
                match self.index {
                    1 => Some(l),
                    2 => Some(r),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl LeftRightNeighbors {
    pub fn iter(&self) -> LeftRightNeighborsIter {
        LeftRightNeighborsIter {
            lr: Some(self.clone()),
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TopNeighbors {
    Normal {
        tl: ChunkIjkVector,
        t: ChunkIjkVector,
        tr: ChunkIjkVector,
    },
    LayerTransition {
        tl: ChunkIjkVector,
        t1: ChunkIjkVector,
        t0: ChunkIjkVector,
        tr: ChunkIjkVector,
    },
    SingleChunkLayerAbove {
        t: ChunkIjkVector,
    },
    MultiChunkLayerAbove {
        chunks: Vec<ChunkIjkVector>,
    },
    TopOfGrid,
}

pub struct TopNeighborsIter {
    top: Option<TopNeighbors>,
    index: usize,
}

impl Iterator for TopNeighborsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.top {
            Some(TopNeighbors::Normal { tl, t, tr }) => {
                self.index += 1;
                match self.index {
                    1 => Some(*tl),
                    2 => Some(*t),
                    3 => Some(*tr),
                    _ => None,
                }
            }
            Some(TopNeighbors::LayerTransition { tl, t0, t1, tr }) => {
                self.index += 1;
                match self.index {
                    1 => Some(*tl),
                    2 => Some(*t1),
                    3 => Some(*t0),
                    4 => Some(*tr),
                    _ => None,
                }
            }
            Some(TopNeighbors::SingleChunkLayerAbove { t }) => {
                self.index += 1;
                match self.index {
                    1 => Some(*t),
                    _ => None,
                }
            }
            Some(TopNeighbors::MultiChunkLayerAbove { chunks }) => {
                if self.index < chunks.len() {
                    self.index += 1;
                    Some(chunks[self.index - 1])
                } else {
                    None
                }
            }
            Some(TopNeighbors::TopOfGrid) => None,
            None => None,
        }
    }
}

impl TopNeighbors {
    pub fn iter(&self) -> TopNeighborsIter {
        TopNeighborsIter {
            top: Some(self.clone()),
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BottomNeighbors {
    Normal {
        bl: ChunkIjkVector,
        b: ChunkIjkVector,
        br: ChunkIjkVector,
    },
    LayerTransition {
        bl: ChunkIjkVector,
        br: ChunkIjkVector,
    },
    FullLayerBelow {
        b: ChunkIjkVector,
    },
    BottomOfGrid,
}

pub struct BottomNeighborsIter {
    bottom: Option<BottomNeighbors>,
    index: usize,
}

impl Iterator for BottomNeighborsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.bottom {
            Some(BottomNeighbors::Normal { bl, b, br }) => {
                self.index += 1;
                match self.index {
                    1 => Some(bl),
                    2 => Some(b),
                    3 => Some(br),
                    _ => None,
                }
            }
            Some(BottomNeighbors::LayerTransition { bl, br }) => {
                self.index += 1;
                match self.index {
                    1 => Some(bl),
                    2 => Some(br),
                    _ => None,
                }
            }
            Some(BottomNeighbors::FullLayerBelow { b }) => {
                self.index += 1;
                match self.index {
                    1 => Some(b),
                    _ => None,
                }
            }
            Some(BottomNeighbors::BottomOfGrid) => None,
            None => None,
        }
    }
}

impl BottomNeighbors {
    pub fn iter(&self) -> BottomNeighborsIter {
        BottomNeighborsIter {
            bottom: Some(self.clone()),
            index: 0,
        }
    }
}

/// Just the indices of the element grid convolution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementGridConvolutionNeighborsChunkIdx {
    pub top: TopNeighbors,
    pub left_right: LeftRightNeighbors,
    pub bottom: BottomNeighbors,
}

pub struct ElementGridConvolutionNeighborsChunkIdxIter {
    top_neighbors_iter: TopNeighborsIter,
    left_right_neighbors_iter: LeftRightNeighborsIter,
    bottom_neighbors_iter: BottomNeighborsIter,
    index: usize,
}

impl Iterator for ElementGridConvolutionNeighborsChunkIdxIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index {
            0 => {
                if let Some(top) = self.top_neighbors_iter.next() {
                    Some(top)
                } else {
                    self.index += 1;
                    self.next()
                }
            }
            1 => {
                if let Some(left_right) = self.left_right_neighbors_iter.next() {
                    Some(left_right)
                } else {
                    self.index += 1;
                    self.next()
                }
            }
            2 => {
                if let Some(bottom) = self.bottom_neighbors_iter.next() {
                    Some(bottom)
                } else {
                    self.index += 1;
                    self.next()
                }
            }
            _ => None,
        }
    }
}

impl ElementGridConvolutionNeighborsChunkIdx {
    pub fn iter(&self) -> ElementGridConvolutionNeighborsChunkIdxIter {
        ElementGridConvolutionNeighborsChunkIdxIter {
            top_neighbors_iter: self.top.iter(),
            left_right_neighbors_iter: self.left_right.iter(),
            bottom_neighbors_iter: self.bottom.iter(),
            index: 0,
        }
    }
    pub fn contains(&self, chunk_idx: &ChunkIjkVector) -> bool {
        self.iter().any(|c| c == *chunk_idx)
    }
}

/// A 3x3 ish grid of element grids
/// However, it's a bit complicated because at the top boundary
/// Also when you go from a single chunk layer to a multi chunk layer
/// And going down a layer you might not have a bottom layer, because you might be at the bottom
/// Also going down a layer you may not have anything below you
/// This has options because you can take stuff from it and give it back
pub struct ElementGridConvolutionNeighbors {
    chunk_idxs: ElementGridConvolutionNeighborsChunkIdx,
    grids: HashMap<ChunkIjkVector, ElementGrid>,
}

/// Instantiation
impl ElementGridConvolutionNeighbors {
    pub fn new(
        chunk_idxs: ElementGridConvolutionNeighborsChunkIdx,
        grids: HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        Self { chunk_idxs, grids }
    }
}

// Into Iter
impl IntoIterator for ElementGridConvolutionNeighbors {
    type Item = (ChunkIjkVector, ElementGrid);
    type IntoIter = std::collections::hash_map::IntoIter<ChunkIjkVector, ElementGrid>;

    fn into_iter(self) -> Self::IntoIter {
        self.grids.into_iter()
    }
}
