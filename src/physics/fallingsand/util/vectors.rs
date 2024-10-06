//! A collection of coordinate types and their conversions
//! Mostly for the [`ChunkCoords`] [`crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir`]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{color::Color, math::Vec2};

use crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords;
use derive_more::{Add, AddAssign, Sub, SubAssign};

/// A coordinate system for  [`ndarray`]
///  [`ndarray`] is row-major, so the jk vector is flipped
/// Top left is (0, 0)
/// ![ndarray coords](../../../../../assets/docs/wireframe/ndarray_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NdArrayCoords([usize; 2]);

/// Instantiation
impl NdArrayCoords {
    /// Create a new  [`NdArrayCoords`]
    pub fn new(x: usize, y: usize) -> Self {
        Self([x, y])
    }
}

impl NdArrayCoords {
    /// Convert to a  [`JkVector`]
    /// ndarray is row-major, so the jk vector is flipped
    /// Bottom Right is (0, 0)
    pub fn to_jk_vector(self, coords: &ChunkCoords) -> JkVector {
        JkVector {
            j: coords.num_concentric_circles() - 1 - self.y(),
            k: coords.num_radial_lines() - 1 - self.x(),
        }
    }

    /// Get the column index
    pub fn x(&self) -> usize {
        self.0[0]
    }

    /// Get the row index
    pub fn y(&self) -> usize {
        self.0[1]
    }
}

impl From<NdArrayCoords> for [usize; 2] {
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
    pub j: usize,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: usize,
}

/// To  [`NdArrayCoords`]
/// ndarray is row-major, so the jk vector is flipped
/// Top left is (0, 0)
/// Whereas in a Jk Vector, the bottom right is (0, 0)
impl JkVector {
    /// Convert to a  [`NdArrayCoords`]
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
    pub fn new(j: usize, k: usize) -> Self {
        Self { j, k }
    }
}

impl From<ChunkIjkVector> for JkVector {
    fn from(value: ChunkIjkVector) -> Self {
        Self {
            j: value.j,
            k: value.k,
        }
    }
}

impl From<RelJkVector> for JkVector {
    fn from(value: RelJkVector) -> Self {
        Self {
            j: value.rj as usize,
            k: value.rk as usize,
        }
    }
}

impl From<IjkVector> for JkVector {
    fn from(value: IjkVector) -> Self {
        Self {
            j: value.j,
            k: value.k,
        }
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
    pub fn new(rj: isize, rk: isize) -> Self {
        Self { rj, rk }
    }
}

impl From<JkVector> for RelJkVector {
    fn from(value: JkVector) -> Self {
        RelJkVector {
            rj: value.j as isize,
            rk: value.k as isize,
        }
    }
}

/// Same as  [`JkVector`] but with i indicating the "layer number"
/// The core is layer 0
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    /// The i coordinate, as in the layer number, the core is 0
    pub i: usize,
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: usize,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: usize,
}

impl IjkVector {
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
    /// Instantiation
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
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
    pub i: usize,
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: usize,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: usize,
}

impl ChunkIjkVector {
    /// Instantiation
    pub fn new(i: usize, j: usize, k: usize) -> Self {
        Self { i, j, k }
    }
    /// The zero vector
    pub const ZERO: Self = Self { i: 0, j: 0, k: 0 };
}
