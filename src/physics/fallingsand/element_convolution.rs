use std::{collections::HashMap, fmt};

use crate::physics::fallingsand::util::vectors::TempJkVector;

use super::{
    coordinates::chunk_coords::ChunkCoords,
    element_grid::ElementGrid,
    elements::element::Element,
    util::{
        functions::modulo,
        vectors::{ChunkIjkVector, FullIdx, JkVector, RelJkVector},
    },
};

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

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone)]
pub struct OutOfBoundsError {
    pub from_chunk: ChunkIjkVector,
    pub naive_idx: TempJkVector,
}
impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} went outside the constraints of chunk {:?}",
            self.naive_idx, self.from_chunk
        )
    }
}

/// Behavior Methods
/// Methods which allow you to get an index "relative" to another index on the element convolution.
/// All of these methods are given "lr priority". Meaning if you ask for rel_idx RelJkVector(-1, -1) that means
/// go clockwise one then go down one. This is easier to program as lr is easy.
impl ElementGridConvolutionNeighbors {
    /// Gets the element one cell below you
    /// TODO: Test this
    pub fn get_below(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
    ) -> Result<FullIdx, OutOfBoundsError> {
        if pos.j > 0 {
            let new_coord = JkVector {
                j: pos.j - 1,
                k: pos.k,
            };
            Ok(FullIdx::new(
                target_chunk.get_chunk_coords().get_chunk_idx(),
                new_coord,
            ))
        } else {
            match self.chunk_idxs.bottom {
                BottomNeighbors::BottomOfGrid => Err(OutOfBoundsError {
                    naive_idx: TempJkVector {
                        j: pos.j as isize - 1,
                        k: pos.k as isize,
                    },
                    from_chunk: target_chunk.get_chunk_coords().get_chunk_idx(),
                }),
                BottomNeighbors::FullLayerBelow { b } => {
                    let bcoords = self.grids.get(&b).unwrap().get_chunk_coords();
                    let new_coords = JkVector {
                        j: bcoords.get_num_radial_lines() - 1,
                        k: pos.k / 2,
                    };
                    Ok(FullIdx::new(b, new_coords))
                }
                BottomNeighbors::LayerTransition { bl, br } => {
                    // TODO: Test this, I forget which way it is
                    if target_chunk.get_chunk_coords().get_chunk_idx().k % 2 == 0 {
                        let blcoords = self.grids.get(&bl).unwrap().get_chunk_coords();
                        let new_coords = JkVector {
                            j: blcoords.get_num_radial_lines() - 1,
                            k: pos.k / 2,
                        };
                        Ok(FullIdx::new(bl, new_coords))
                    } else {
                        let brcoords = self.grids.get(&br).unwrap().get_chunk_coords();
                        let new_coords = JkVector {
                            j: brcoords.get_num_radial_lines() - 1,
                            k: pos.k / 2,
                        };
                        Ok(FullIdx::new(br, new_coords))
                    }
                }
                BottomNeighbors::Normal { bl: _, b, br: _ } => {
                    let bcoords = self.grids.get(&b).unwrap().get_chunk_coords();
                    let new_coords = JkVector {
                        j: bcoords.get_num_radial_lines() - 1,
                        k: pos.k / 2,
                    };
                    Ok(FullIdx::new(b, new_coords))
                }
            }
        }
    }
}
