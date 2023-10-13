use std::collections::{HashMap, HashSet};

use super::{
    element_grid::ElementGrid,
    elements::element::Element,
    util::vectors::{ChunkIjkVector, IjkVector, RelJkVector},
};

/// Just the indices of the element grid convolution
#[derive(Clone)]
pub struct ElementGridConvolutionChunkIdx {
    pub center: ChunkIjkVector,
    pub neighbors: HashSet<ChunkIjkVector>,
}

/// Instantiation
impl ElementGridConvolutionChunkIdx {
    pub fn new(center: ChunkIjkVector) -> Self {
        Self {
            center,
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
    center: Option<ElementGrid>,
    neighbors: Option<HashMap<ChunkIjkVector, ElementGrid>>,
}

/// We implement IntoIterator for ElementGridConvolution so that we can unpackage it
/// back into a element grid directory
pub struct IntoIter {
    convolution: ElementGridConvolution,
    position: usize,
}

/// Instantiation
impl ElementGridConvolution {
    pub fn new(center: ElementGrid) -> Self {
        Self {
            center: Some(center),
            neighbors: Some(HashMap::new()),
        }
    }
}

/// Getters
impl ElementGridConvolution {
    pub fn get_center(&self) -> &ElementGrid {
        &self.center.as_ref().unwrap()
    }
    pub fn get_center_mut(&mut self) -> &mut ElementGrid {
        self.center.as_mut().unwrap()
    }
    pub fn take_center(&mut self) -> ElementGrid {
        self.center.take().unwrap()
    }
    pub fn set_center(&mut self, center: ElementGrid) {
        self.center = Some(center);
    }
    pub fn get_neighbors(&self) -> &HashMap<ChunkIjkVector, ElementGrid> {
        &self.neighbors.as_ref().unwrap()
    }
    pub fn get_neighbors_mut(&mut self) -> &mut HashMap<ChunkIjkVector, ElementGrid> {
        self.neighbors.as_mut().unwrap()
    }
    pub fn take_neighbors(&mut self) -> HashMap<ChunkIjkVector, ElementGrid> {
        self.neighbors.take().unwrap()
    }
    pub fn set_neighbors(&mut self, neighbors: HashMap<ChunkIjkVector, ElementGrid>) {
        self.neighbors = Some(neighbors);
    }
}

/// Complex Indexed Getters & Setters
impl ElementGridConvolution {
    pub fn get_idx(&self, idx: IjkVector) -> Option<&ElementGrid> {
        unimplemented!();
    }
    pub fn get_idx_mut(&mut self, idx: IjkVector) -> Option<&mut ElementGrid> {
        unimplemented!();
    }
    pub fn replace_idx(
        &mut self,
        idx: IjkVector,
        element: Box<dyn Element>,
    ) -> Option<Box<dyn Element>> {
        unimplemented!();
    }
    /// Set the index, but don't return the old value
    pub fn set_idx(&mut self, idx: IjkVector, element: Box<dyn Element>) {
        let _ = self.replace_idx(idx, element);
    }
}

/// Index Transforms
impl ElementGridConvolution {
    pub fn transform_rel_ijkvector_to_ijkvector(
        &self,
        movement: RelJkVector,
        pos: IjkVector,
    ) -> (Option<IjkVector>, Option<IjkVector>) {
        unimplemented!();
    }
}
