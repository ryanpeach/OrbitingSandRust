//! Indexes in [ChunkIjkVector]s for all the neighbors of a chunk
use crate::physics::fallingsand::{mesh::chunk_coords::ChunkCoords, util::vectors::ChunkIjkVector};

/// The main type exported by this module
/// Contains all the [ChunkIjkVector] indexes for the convolution
/// Check out the [super::neighbor_identifiers::ConvolutionIdentifier] and
/// [super::neighbor_indexes::ElementGridConvolutionNeighborIdxs] documentation for more information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElementGridConvolutionNeighborIdxs {
    /// Top neighbor indexes
    pub top: TopNeighborIdxs,
    /// Left and Right neighbor indexes
    pub left_right: LeftRightNeighborIdxs,
    /// Bottom neighbor indexes
    pub bottom: BottomNeighborIdxs,
}

/// An iterator for the neighbor indexes
pub struct ElementGridConvolutionNeighborIdxsIter {
    /// The iterator for the top neighbor indexes
    top_neighbors_iter: TopNeighborIdxsIter,
    /// The iterator for the left and right neighbor indexes
    left_right_neighbors_iter: LeftRightNeighborIdxsIter,
    /// The iterator for the bottom neighbor indexes
    bottom_neighbors_iter: BottomNeighborIdxsIter,
    /// The current index
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
    /// Get the iterator for the neighbor indexes
    pub fn iter(&self) -> ElementGridConvolutionNeighborIdxsIter {
        ElementGridConvolutionNeighborIdxsIter {
            top_neighbors_iter: self.top.iter(),
            left_right_neighbors_iter: self.left_right.iter(),
            bottom_neighbors_iter: self.bottom.iter(),
            index: 0,
        }
    }

    /// Check if the given [ChunkIjkVector] is contained in the neighbor indexes
    pub fn contains(&self, chunk_idx: &ChunkIjkVector) -> bool {
        self.iter().any(|c| c == *chunk_idx)
    }
}

/// Left and Right neighbor indexes in the convolution
/// Check out the [super::neighbor_identifiers::LeftRightNeighborIdentifier] and
/// [super::neighbor_grids::LeftRightNeighborGrids] documentation for more information
/// documentation for more information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LeftRightNeighborIdxs {
    /// The left and right elements
    /// TODO: Unecessary to have a struct for this, flatten into the enum
    LR {
        /// Left element
        l: ChunkIjkVector,
        /// Right element
        r: ChunkIjkVector,
    },
}

/// An iterator for the left and right neighbor indexes
pub struct LeftRightNeighborIdxsIter {
    /// Whether or not there are left and right neighbors
    /// TODO: Shouldnt this always be Some?
    lr: Option<LeftRightNeighborIdxs>,
    /// The current index
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
    /// Get the iterator for the left and right neighbor indexes
    pub fn iter(&self) -> LeftRightNeighborIdxsIter {
        LeftRightNeighborIdxsIter {
            lr: Some(self.clone()),
            index: 0,
        }
    }
}

/// Top neighbor indexes in the convolution
/// Check out the [super::neighbor_identifiers::TopNeighborIdentifier] and
/// [super::neighbor_grids::TopNeighborGrids] documentation for more information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TopNeighborIdxs {
    /// Indicates that there are the same number of chunks above as you have
    /// However, the cells may still double tangentially
    /// However, some enums need that information and others dont, so we
    /// dont want to add too much complexity to the match statements if we can help it
    Normal {
        /// Top left element
        tl: ChunkIjkVector,
        /// Top center element
        t: ChunkIjkVector,
        /// Top right element
        tr: ChunkIjkVector,
    },
    /// Indicates a **chunk doubling** layer transition
    ChunkDoubling {
        /// Top left element
        tl: ChunkIjkVector,
        /// Second top center element, left of center
        t1: ChunkIjkVector,
        /// First top center element, right of center
        t0: ChunkIjkVector,
        /// Top right element
        tr: ChunkIjkVector,
    },
    /// Indicates that you are at the top of the grid
    TopOfGrid,
}

/// An iterator for the top neighbor indexes
pub struct TopNeighborIdxsIter {
    /// Whether or not there are top neighbors
    top: Option<TopNeighborIdxs>,
    /// The current index
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
    /// Get the iterator for the top neighbor indexes
    pub fn iter(&self) -> TopNeighborIdxsIter {
        TopNeighborIdxsIter {
            top: Some(self.clone()),
            index: 0,
        }
    }
}

/// Bottom neighbor indexes in the convolution
/// Check out the [super::neighbor_identifiers::BottomNeighborIdentifier] and
/// [super::neighbor_grids::BottomNeighborGrids] documentation for more information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BottomNeighborIdxs {
    /// Indicates that there are the same number of chunks below as you have
    /// However, the cells may still half tangentially
    /// However, some enums need that information and others dont, so we
    /// dont want to add too much complexity to the match statements if we can help it
    Normal {
        /// The bottom left chunk
        bl: ChunkIjkVector,
        /// The bottom chunk
        b: ChunkIjkVector,
        /// The bottom right chunk
        br: ChunkIjkVector,
    },
    /// Indicates a **chunk doubling** layer transition
    /// One of these will be directly below you, and be bigger than you off to one direction
    /// Whereas the other will be diagonally below you
    /// This depends on if your [ChunkIjkVector] has a `k` value which is even or odd
    /// If it is even, then the `bl` will be directly below you, and you will be straddling its right side
    /// If it is odd, then the `br` will be directly below you, and you will be straddling its left side
    ChunkDoubling {
        /// The bottom left chunk
        bl: ChunkIjkVector,
        /// The bottom right chunk
        br: ChunkIjkVector,
    },
    /// Indicates that you are at the bottom of the grid
    BottomOfGrid,
}

/// An iterator for the bottom neighbor indexes
pub struct BottomNeighborIdxsIter {
    /// Whether or not there are bottom neighbors
    bottom: Option<BottomNeighborIdxs>,
    /// The current index
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
    /// Get the iterator for the bottom neighbor indexes
    pub fn iter(&self) -> BottomNeighborIdxsIter {
        BottomNeighborIdxsIter {
            bottom: Some(self.clone()),
            index: 0,
        }
    }

    /// Determine which is the actual "bottom" chunk
    /// *coords* is the [ChunkCoords] of the chunk you are in
    /// If it is even, then the `bl` will be directly below you, and you will be straddling its right side
    /// If it is odd, then the `br` will be directly below you, and you will be straddling its left side
    pub fn get_bottom_chunk(&self, coords: ChunkCoords) -> Option<ChunkIjkVector> {
        match self {
            BottomNeighborIdxs::Normal { b, .. } => Some(*b),
            BottomNeighborIdxs::ChunkDoubling { bl, br } => {
                if coords.get_chunk_idx().k % 2 == 0 {
                    Some(*bl)
                } else {
                    Some(*br)
                }
            }
            BottomNeighborIdxs::BottomOfGrid => None,
        }
    }
}
