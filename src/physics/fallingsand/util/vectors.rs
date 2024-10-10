//! A collection of coordinate types and their conversions
//! Mostly for the [`ChunkCoords`] [`crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir`]
#![warn(missing_docs)]

use bevy::{color::Color, math::Vec2};

use crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords;
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// A coordinate system for  [`ndarray`]
///  [`ndarray`] is row-major, so the jk vector is flipped
/// Top left is (0, 0)
/// ![ndarray coords](../../../../../assets/docs/wireframe/ndarray_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NdArrayCoords([u32; 2]);

/// Instantiation
impl NdArrayCoords {
    /// Create a new  [`NdArrayCoords`]
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Self([x, y])
    }
}

impl NdArrayCoords {
    /// Convert to a  [`JkVector`]
    /// ndarray is row-major, so the jk vector is flipped
    /// Bottom Right is (0, 0)
    #[must_use]
    pub fn to_jk_vector(self, coords: &ChunkCoords) -> JkVector {
        JkVector {
            j: coords.num_concentric_circles() - 1 - self.y(),
            k: coords.num_radial_lines() - 1 - self.x(),
        }
    }

    /// Get the column index
    #[must_use]
    pub fn x(&self) -> u32 {
        self.0[0]
    }

    /// Get the row index
    #[must_use]
    pub fn y(&self) -> u32 {
        self.0[1]
    }
}

impl From<NdArrayCoords> for [u32; 2] {
    fn from(val: NdArrayCoords) -> Self {
        val.0
    }
}

/// Constants
impl NdArrayCoords {
    /// The zero vector
    pub const ZERO: Self = Self([0, 0]);
}

/// My personal coordinate type for the circular grids
/// basically radius-theta coordinates, with integer radius and theta
/// "counter clockwise" is positive just like in the unit circle
///
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
///
/// j is the "concentric circle" or "radial" axis, kinda like y,
///   towards the core is 0
/// k is the "tangential" axis, kinda like x,
///   positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
///
/// Can also be used to describe a grid, like a chunk taken from the circle
/// In this case j is the height and k is the width
/// Bottom right is (0, 0)
/// If you need to also know the layer number, use  [`IjkVector`]
/// If you need a relative vector, use  [`RelJkVector`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct JkVector {
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: u32,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: u32,
}

/// To  [`NdArrayCoords`]
/// ndarray is row-major, so the jk vector is flipped
/// Top left is (0, 0)
/// Whereas in a Jk Vector, the bottom right is (0, 0)
impl JkVector {
    /// Convert to a  [`NdArrayCoords`]
    #[must_use]
    pub fn to_ndarray_coords(self, coords: &ChunkCoords) -> NdArrayCoords {
        NdArrayCoords::new(
            coords.num_radial_lines() - 1 - self.k,
            coords.num_concentric_circles() - 1 - self.j,
        )
    }
}

/// Convienient constants
impl JkVector {
    /// The zero vector
    pub const ZERO: Self = Self { j: 0, k: 0 };
}

/// Instantiation
impl JkVector {
    /// Create a new  [`JkVector`]
    #[must_use]
    pub fn new(j: u32, k: u32) -> Self {
        Self { j, k }
    }
}

/// This defines a movement or a vector relative to some position on the circular grid
/// Same as  [`JkVector`], but with isize type fields which can contain negative numbers
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelJkVector {
    /// The relative j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub rj: isize,
    /// The relative k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub rk: isize,
}

/// Instantiation
impl RelJkVector {
    /// Create a new  [`RelJkVector`]
    #[must_use]
    pub fn new(rj: isize, rk: isize) -> Self {
        Self { rj, rk }
    }
}

/// Sometimes while resolving a relative  [`JkVector`] into a  [`JkVector`] when you
/// need isize type fields
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TempJkVector {
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: isize,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: isize,
}

/// Instantiation
impl TempJkVector {
    /// Add a  [`RelJkVector`]to a  [`JkVector`]
    #[must_use]
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
    /// The chunk index
    pub chunk_idx: ChunkIjkVector,
    /// The position of an element within a chunk
    pub pos: JkVector,
}

/// Instantiation
impl FullIdx {
    /// Create a new [`FullIdx`]
    #[must_use]
    pub fn new(chunk_idx: ChunkIjkVector, pos: JkVector) -> Self {
        Self { chunk_idx, pos }
    }
}

/// Same as  [`JkVector`] but with i indicating the "layer number"
/// The core is layer 0
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    /// The i coordinate, as in the layer number, the core is 0
    pub i: u32,
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: u32,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: u32,
}

impl IjkVector {
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    /// Instantiation
    #[must_use]
    pub fn new(i: u32, j: u32, k: u32) -> Self {
        Self { i, j, k }
    }
    /// Convert to a [`JkVector`]
    #[must_use]
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
    /// The position of the vertex
    pub position: Vec2,
    /// The texture coordinates of the vertex
    pub uv: Vec2,
    /// The color of the vertex
    pub color: Color,
}

/// The  [`IjkVector`]of a chunk within an [`crate::physics::fallingsand::data::element_directory::ElementGridDir`]
/// In this case Ijk relate to the index of the chunk itself, not
/// perportional to the cells within the chunk
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkIjkVector {
    /// The i coordinate, as in the layer number, the core is 0
    pub i: u32,
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: u32,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: u32,
}

impl ChunkIjkVector {
    /// Instantiation
    #[must_use]
    pub fn new(i: u32, j: u32, k: u32) -> Self {
        Self { i, j, k }
    }
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    /// Convert to a [`JkVector`]
    #[must_use]
    pub fn to_jk_vector(self) -> JkVector {
        JkVector {
            j: self.j,
            k: self.k,
        }
    }
}
