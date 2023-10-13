use std::collections::{HashMap, HashSet};

use super::{element_grid::ElementGrid, util::vectors::ChunkIjkVector};

/// Just the indices of the element grid convolution
#[derive(Clone)]
pub struct ElementGridConvolutionChunkIdx {
    pub neighbors: HashSet<ChunkIjkVector>,
}

/// Instantiation
impl ElementGridConvolutionChunkIdx {
    pub fn new() -> Self {
        Self {
            neighbors: HashSet::new(),
        }
    }
}

/// A 3x3 grid of element grids
/// However, it's a bit complicated because at the top boundary
/// you might encounter a doubling of the grid size, in the case where you are going up
/// a level, that's why there is a t1 and t2.
/// Or at the very top level all the upper levels might be None
/// And going down a layer you might not have a bottom layer, because you might be at the bottom
/// Also going down a layer you may not have a b, because you would only have a bl or br
/// This has options because you can take stuff from it and give it back
pub struct ElementGridConvolution {
    neighbors: HashMap<ChunkIjkVector, ElementGrid>,
}

/// We implement IntoIterator for ElementGridConvolution so that we can unpackage it
/// back into a element grid directory
pub struct IntoIter {
    convolution: ElementGridConvolution,
    position: usize,
}

/// Instantiation
impl ElementGridConvolution {
    pub fn new() -> Self {
        Self {
            neighbors: HashMap::new(),
        }
    }
}

/// Getters
impl ElementGridConvolution {
    pub fn get_neighbors(&self) -> &HashMap<ChunkIjkVector, ElementGrid> {
        &self.neighbors
    }
    pub fn get_neighbors_mut(&mut self) -> &mut HashMap<ChunkIjkVector, ElementGrid> {
        &mut self.neighbors
    }
    pub fn take_neighbors(self) -> HashMap<ChunkIjkVector, ElementGrid> {
        self.neighbors
    }
    pub fn set_neighbors(&mut self, neighbors: HashMap<ChunkIjkVector, ElementGrid>) {
        self.neighbors = neighbors;
    }
}
