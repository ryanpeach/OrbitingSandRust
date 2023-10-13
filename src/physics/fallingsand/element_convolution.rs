use std::collections::{HashMap, HashSet};

use super::{element_grid::ElementGrid, util::vectors::ChunkIjkVector};

/// Just the indices of the element grid convolution
pub type ElementGridConvolutionChunkIdx = HashSet<ChunkIjkVector>;

/// A 3x3 ish grid of element grids
/// However, it's a bit complicated because at the top boundary
/// Also when you go from a single chunk layer to a multi chunk layer
/// And going down a layer you might not have a bottom layer, because you might be at the bottom
/// Also going down a layer you may not have anything below you
/// This has options because you can take stuff from it and give it back
pub type ElementGridConvolution = HashMap<ChunkIjkVector, ElementGrid>;
