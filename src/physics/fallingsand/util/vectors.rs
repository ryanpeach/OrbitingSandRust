//! A collection of coordinate types and their conversions
//! Mostly for the [ChunkCoords] [crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{math::Vec2, render::color::Color};

use crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords;
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// A coordinate system for [ndarray]
/// [ndarray] is row-major, so the jk vector is flipped
/// Top left is (0, 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct NdArrayCoords {
    pub x: usize,
    pub y: usize,
}

/// Instantiation
impl NdArrayCoords {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl NdArrayCoords {
    /// Convert to a [JkVector]
    /// ndarray is row-major, so the jk vector is flipped
    /// Bottom Right is (0, 0)
    pub fn to_jk_vector(self, coords: &ChunkCoords) -> JkVector {
        JkVector {
            j: coords.get_num_concentric_circles() - 1 - self.y,
            k: coords.get_num_radial_lines() - 1 - self.x,
        }
    }
}

impl Into<[usize; 2]> for NdArrayCoords {
    fn into(self) -> [usize; 2] {
        [self.x, self.y]
    }
}

/// Constants
impl NdArrayCoords {
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

/// My personal coordinate type for the circular grids
/// basically radius-theta coordinates, with integer radius and theta
/// "counter clockwise" is positive just like in the unit circle
/// j is the "concentric circle" or "radial" axis, kinda like y,
///   towards the core is 0
/// k is the "tangential" axis, kinda like x,
///   positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
/// Can also be used to describe a grid, like a chunk taken from the circle
/// In this case j is the height and k is the width
/// Bottom right is (0, 0)
/// If you need to also know the layer number, use [IjkVector]
/// If you need a relative vector, use [RelJkVector]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct JkVector {
    pub j: usize,
    pub k: usize,
}

/// To [NdArrayCoords]
/// ndarray is row-major, so the jk vector is flipped
/// Top left is (0, 0)
/// Whereas in a Jk Vector, the bottom right is (0, 0)
impl JkVector {
    pub fn to_ndarray_coords(self, coords: &ChunkCoords) -> NdArrayCoords {
        NdArrayCoords::new(
            coords.get_num_radial_lines() - 1 - self.k,
            coords.get_num_concentric_circles() - 1 - self.j,
        )
    }
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

/// This defines a movement or a vector relative to some position on the circular grid
/// Same as [JkVector], but with isize type fields which can contain negative numbers
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

/// Sometimes while resolving a relative [JkVector] into a [JkVector] when you
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

/// Same as [JkVector], but with i indicating the "layer number"
/// The core is layer 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

impl IjkVector {
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    /// Instantiation
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
    }
    /// Convert to a [JkVector]
    pub fn to_jk_vector(self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}

/// A vertex in a mesh
/// Originally from ggez
/// TODO: move to bevy's types
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

/// The [IjkVector] of a chunk within an [crate::physics::fallingsand::data::element_directory::ElementGridDir]
/// In this case Ijk relate to the index of the chunk itself, not
/// perportional to the cells within the chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkIjkVector {
    pub i: usize,
    pub j: usize,
    pub k: usize,
}

impl ChunkIjkVector {
    /// Instantiation
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
    }
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    /// Convert to a [JkVector]
    pub fn to_jk_vector(self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}
