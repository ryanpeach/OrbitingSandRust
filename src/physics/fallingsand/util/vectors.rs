/// My personal coordinate type
/// j is the "concentric circle" axis, kinda like y,
///   towards the core is 0
/// k is the "radial line" axis, kinda like x,
///   positive is counter clockwise from unit circle 0 degrees
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JkVector {
    pub j: usize,
    pub k: usize,
}

/// Same as JkVector, but with i indicating the "layer number"
/// The core is layer 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

/// The ijk coordinates of a chunk within an element grid directory
/// In this case Ijk relate to the index of the chunk itself, not
/// perportional to the cells within the chunk
/// Eg: The chunk on the 3rd layer, two chunks up and one chunk around would be
/// > ChunkIjkVector { i: 3, j: 2, k: 1 }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkIjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

/// Convienient constants
impl ChunkIjkVector {
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
}

/// Convienient conversions between coordinate types
impl ChunkIjkVector {
    pub fn to_jk_vector(&self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}
