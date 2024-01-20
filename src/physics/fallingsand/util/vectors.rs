use bevy::{math::Vec2, render::color::Color};

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

/// Convienient constants
impl JkVector {
    pub const ZERO: Self = Self { j: 0, k: 0 };
}

/// Instantiation
impl JkVector {
    pub fn new(j: usize, k: usize) -> Self {
        Self { j, k }
    }
}

/// This defines a movement or a vector relative to some position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelJkVector {
    pub rj: isize,
    pub rk: isize,
}

/// Instantiation
impl RelJkVector {
    pub fn new(rj: isize, rk: isize) -> Self {
        Self { rj, rk }
    }
}

/// Sometimes while resolving a relative JKVector into a JKVector you
/// need isize type fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TempJkVector {
    pub j: isize,
    pub k: isize,
}

/// Instantiation
impl TempJkVector {
    pub fn add(pos: &JkVector, rel: &RelJkVector) -> Self {
        Self {
            j: pos.j as isize + rel.rj,
            k: pos.k as isize + rel.rk,
        }
    }
}

/// Defines both the chunk and the internal idx of the element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FullIdx {
    pub chunk_idx: ChunkIjkVector,
    pub pos: JkVector,
}

/// Instantiation
impl FullIdx {
    pub fn new(chunk_idx: ChunkIjkVector, pos: JkVector) -> Self {
        Self { chunk_idx, pos }
    }
}

/// Same as JkVector, but with i indicating the "layer number"
/// The core is layer 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

/// Convienient constants
impl IjkVector {
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
    }
    pub fn to_jk_vector(self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}

/// A vertex in a mesh
/// Originally from ggez
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

/// A rectangle
/// Originally from ggez
/// TODO: Replace with bevy::math::Rect
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The ijk coordinates of a chunk within an element grid directory
/// In this case Ijk relate to the index of the chunk itself, not
/// perportional to the cells within the chunk
/// Eg: The chunk on the 3rd layer, two chunks up and one chunk around would be
/// > ChunkIjkVector { i: 3, j: 2, k: 1 }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkIjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

/// Instantiation
impl ChunkIjkVector {
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
    }
}

/// Convienient constants
impl ChunkIjkVector {
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
}

/// Convienient conversions between coordinate types
impl ChunkIjkVector {
    pub fn to_jk_vector(self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}
