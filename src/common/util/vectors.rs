//! A collection of coordinate types and their conversions
//! Mostly for the [`ChunkCoords`] [`crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir`]
#![warn(missing_docs)]

use bevy::{
    math::Vec2,
    transform::components::{GlobalTransform, Transform},
};

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
/// Gives all the different kinds of jk vectors some common functions
pub trait JkVector
where
    Self: Sized,
{
    /// Initialization
    #[must_use]
    fn new(j: u32, k: u32) -> Self;
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    #[must_use]
    fn j(&self) -> u32;
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    #[must_use]
    fn k(&self) -> u32;
    /// To  [`NdArrayCoords`]
    /// ndarray is row-major, so the jk vector is flipped
    /// Top left is (0, 0)
    /// Whereas in a Jk Vector, the bottom right is (0, 0)
    #[must_use]
    fn to_ndarray_coords(self, coords: &ChunkCoords) -> NdArrayCoords {
        NdArrayCoords::new(
            coords.num_radial_lines() - 1 - self.k(),
            coords.num_concentric_circles() - 1 - self.j(),
        )
    }
}

/// The difference between this and [`InChunkJkVector`]
/// is that this assumes you are indexing within the entire layer.
/// This is a cell index, NOT a chunk index
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct LayerJkVector {
    /// See [`JkVector::j`]
    pub j: u32,
    /// See [`JkVector::k`]
    pub k: u32,
}

impl JkVector for LayerJkVector {
    fn new(j: u32, k: u32) -> Self {
        Self { j, k }
    }
    fn j(&self) -> u32 {
        self.j
    }
    fn k(&self) -> u32 {
        self.k
    }
}

/// This is for the coordinates of the **chunk itself** within a layer.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct ChunkJkVector {
    /// See [`JkVector::j`]
    pub j: u32,
    /// See [`JkVector::k`]
    pub k: u32,
}

impl ChunkJkVector {
    /// Convert to a  [`NdArrayCoords`]
    #[must_use]
    pub fn to_ndarray_coords(self, coords: &ChunkCoords) -> NdArrayCoords {
        NdArrayCoords::new(
            coords.num_radial_lines() - 1 - self.k,
            coords.num_concentric_circles() - 1 - self.j,
        )
    }
}

impl JkVector for ChunkJkVector {
    fn new(j: u32, k: u32) -> Self {
        Self { j, k }
    }
    fn j(&self) -> u32 {
        self.j
    }
    fn k(&self) -> u32 {
        self.k
    }
}

/// See [`ChunkJkVector`] but this is for coordinates of a **cell** within a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct InChunkJkVector {
    /// See [`JkVector::j`]
    pub j: u32,
    /// See [`JkVector::k`]
    pub k: u32,
}

impl JkVector for InChunkJkVector {
    fn new(j: u32, k: u32) -> Self {
        Self { j, k }
    }
    fn j(&self) -> u32 {
        self.j
    }
    fn k(&self) -> u32 {
        self.k
    }
}

/// This defines a movement or a vector relative to some position on the circular grid
/// Same as  [`JkVector`], but with isize type fields which can contain negative numbers
/// ![jk vector](../../../../../assets/docs/wireframe/jk_coords.png)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelJkVector {
    /// The relative j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive.
    /// See [`JkVector::j`]
    pub rj: isize,
    /// The relative k coordinate, as in the tangential dimension,
    /// positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    /// See [`JkVector::k`]
    pub rk: isize,
}

/// Defines both the chunk and the internal idx of the element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FullIdx {
    /// The chunk index
    pub chunk_idx: ChunkIjkVector,
    /// The position of an element within a chunk
    pub pos: InChunkJkVector,
}

/// Instantiation
impl FullIdx {
    /// Create a new [`FullIdx`]
    #[must_use]
    pub fn new(chunk_idx: ChunkIjkVector, pos: InChunkJkVector) -> Self {
        Self { chunk_idx, pos }
    }
}

/// Same as  [`JkVector`] but with i indicating the "layer number"
///
/// The core is layer 0
///
/// This particular class is in reference to an actual **cell** inside a [`crate::physics::fallingsand::data::element_directory::ElementGridDir`]
/// or [`crate::physics::fallingsand::mesh::coordinate_dir::CoordinateDir`],
/// NOT the index of the chunk itself, that is a [`ChunkIjkVector`]
///
/// You can consider this the "Absolute coordinates" of a celestial
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IjkVector {
    /// The i coordinate, as in the layer number, the core is 0
    pub i: u32,
    /// The j coordinate, as in the radial dimension, towards the core is negative, away from the core is positive
    pub j: u32,
    /// The k coordinate, as in the tangential dimension, positive is counter clockwise from unit circle 0 degrees which is starting from 3 o'clock east
    pub k: u32,
}

impl IjkVector {
    /// Instantiation
    #[must_use]
    pub fn new(i: u32, j: u32, k: u32) -> Self {
        Self { i, j, k }
    }
    /// Convert to a [`ChunkJkVector`]
    #[must_use]
    pub fn to_chunk_jk_vector(self) -> ChunkJkVector {
        ChunkJkVector {
            j: self.j,
            k: self.k,
        }
    }
}

/// The  [`IjkVector`] of a chunk within a [`crate::physics::fallingsand::data::element_directory::ElementGridDir`]
/// In this case Ijk relate to the index of the chunk itself, not
/// perportional to the cells within the chunk. That would be a standard [`IjkVector`]
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

    /// Just cop off the layer
    #[must_use]
    pub fn to_jk_vector(&self) -> ChunkJkVector {
        ChunkJkVector::new(self.j, self.k)
    }
}

/// An xy vector that is relative to some entity.
/// Using `Model` in accorance with [Bevy Matrix
/// Names](https://bevyengine.org/news/bevy-0-14/#improved-matrix-naming)
#[derive(Debug, Copy, Clone, PartialEq, Sub, Add)]
pub struct ModelCoord(pub Vec2);

impl From<ModelCoord> for Transform {
    fn from(val: ModelCoord) -> Self {
        Transform::from_translation(bevy::math::Vec3 {
            x: val.0.x,
            y: val.0.y,
            z: 0.0,
        })
    }
}

impl ModelCoord {
    /// Initialization
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

/// Coordinates in the cameras view
///
/// Using `View` in accorance with [Bevy Matrix
/// Names](https://bevyengine.org/news/bevy-0-14/#improved-matrix-naming)
#[derive(Debug, Copy, Clone, PartialEq, Sub, Add)]
pub struct ViewCoord(pub Vec2);

impl From<ViewCoord> for Transform {
    fn from(val: ViewCoord) -> Self {
        Transform::from_translation(bevy::math::Vec3 {
            x: val.0.x,
            y: val.0.y,
            z: 0.0,
        })
    }
}

impl ViewCoord {
    /// Initialization
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

/// Global coordinates in the world
///
/// Using `World` in accorance with [Bevy Matrix
/// Names](https://bevyengine.org/news/bevy-0-14/#improved-matrix-naming)
#[derive(Debug, Copy, Clone, PartialEq, Sub, Add)]
pub struct WorldCoord(pub Vec2);

impl From<WorldCoord> for Transform {
    fn from(val: WorldCoord) -> Self {
        Self::from_translation(bevy::math::Vec3 {
            x: val.0.x,
            y: val.0.y,
            z: 0.0,
        })
    }
}

impl From<WorldCoord> for GlobalTransform {
    fn from(val: WorldCoord) -> Self {
        Self::from_translation(bevy::math::Vec3 {
            x: val.0.x,
            y: val.0.y,
            z: 0.0,
        })
    }
}

impl From<Vec2> for WorldCoord {
    fn from(val: Vec2) -> Self {
        Self(val)
    }
}

impl From<ModelCoord> for WorldCoord {
    fn from(val: ModelCoord) -> Self {
        Self(val.0)
    }
}

impl From<WorldCoord> for ModelCoord {
    fn from(val: WorldCoord) -> Self {
        Self(val.0)
    }
}

impl WorldCoord {
    /// Initialization
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}
