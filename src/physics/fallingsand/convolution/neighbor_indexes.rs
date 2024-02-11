//! Indexes in [ChunkIjkVector]s for all the neighbors of a chunk
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;

/// Just the indices of the element grid convolution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementGridConvolutionNeighborIdxs {
    pub top: TopNeighborIdxs,
    pub left_right: LeftRightNeighborIdxs,
    pub bottom: BottomNeighborIdxs,
}

pub struct ElementGridConvolutionNeighborIdxsIter {
    top_neighbors_iter: TopNeighborIdxsIter,
    left_right_neighbors_iter: LeftRightNeighborIdxsIter,
    bottom_neighbors_iter: BottomNeighborIdxsIter,
    index: usize,
}

impl Iterator for ElementGridConvolutionNeighborIdxsIter {
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

impl ElementGridConvolutionNeighborIdxs {
    pub fn iter(&self) -> ElementGridConvolutionNeighborIdxsIter {
        ElementGridConvolutionNeighborIdxsIter {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeftRightNeighborIdxs {
    LR {
        l: ChunkIjkVector,
        r: ChunkIjkVector,
    },
}

pub struct LeftRightNeighborIdxsIter {
    lr: Option<LeftRightNeighborIdxs>,
    index: usize,
}

impl Iterator for LeftRightNeighborIdxsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lr {
            Some(LeftRightNeighborIdxs::LR { l, r }) => {
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

impl LeftRightNeighborIdxs {
    pub fn iter(&self) -> LeftRightNeighborIdxsIter {
        LeftRightNeighborIdxsIter {
            lr: Some(self.clone()),
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TopNeighborIdxs {
    Normal {
        tl: ChunkIjkVector,
        t: ChunkIjkVector,
        tr: ChunkIjkVector,
    },
    ChunkDoubling {
        tl: ChunkIjkVector,
        t1: ChunkIjkVector,
        t0: ChunkIjkVector,
        tr: ChunkIjkVector,
    },
    TopOfGrid,
}

pub struct TopNeighborIdxsIter {
    top: Option<TopNeighborIdxs>,
    index: usize,
}

impl Iterator for TopNeighborIdxsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.top {
            Some(TopNeighborIdxs::Normal { tl, t, tr }) => {
                self.index += 1;
                match self.index {
                    1 => Some(*tl),
                    2 => Some(*t),
                    3 => Some(*tr),
                    _ => None,
                }
            }
            Some(TopNeighborIdxs::ChunkDoubling { tl, t0, t1, tr }) => {
                self.index += 1;
                match self.index {
                    1 => Some(*tl),
                    2 => Some(*t1),
                    3 => Some(*t0),
                    4 => Some(*tr),
                    _ => None,
                }
            }
            Some(TopNeighborIdxs::TopOfGrid) => None,
            None => None,
        }
    }
}

impl TopNeighborIdxs {
    pub fn iter(&self) -> TopNeighborIdxsIter {
        TopNeighborIdxsIter {
            top: Some(self.clone()),
            index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BottomNeighborIdxs {
    Normal {
        bl: ChunkIjkVector,
        b: ChunkIjkVector,
        br: ChunkIjkVector,
    },
    /// Indicates a **chunk doubling** layer transition
    /// In this case the chunks half because you are going down
    ChunkDoubling {
        bl: ChunkIjkVector,
        br: ChunkIjkVector,
    },
    BottomOfGrid,
}

pub struct BottomNeighborIdxsIter {
    bottom: Option<BottomNeighborIdxs>,
    index: usize,
}

impl Iterator for BottomNeighborIdxsIter {
    type Item = ChunkIjkVector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.bottom {
            Some(BottomNeighborIdxs::Normal { bl, b, br }) => {
                self.index += 1;
                match self.index {
                    1 => Some(bl),
                    2 => Some(b),
                    3 => Some(br),
                    _ => None,
                }
            }
            Some(BottomNeighborIdxs::ChunkDoubling { bl, br }) => {
                self.index += 1;
                match self.index {
                    1 => Some(bl),
                    2 => Some(br),
                    _ => None,
                }
            }
            Some(BottomNeighborIdxs::BottomOfGrid) => None,
            None => None,
        }
    }
}

impl BottomNeighborIdxs {
    pub fn iter(&self) -> BottomNeighborIdxsIter {
        BottomNeighborIdxsIter {
            bottom: Some(self.clone()),
            index: 0,
        }
    }
}
