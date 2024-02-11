//! Identifiers for the different locations chunks can be in the convolution
use strum_macros::EnumIter;

use crate::physics::fallingsand::util::vectors::JkVector;

/// The main type exported by this module
/// An enum that identifies the location in the structure of the convolution
/// Check out the [super::neighbor_grids::ElementGridConvolutionNeighborGrids] and
/// [super::neighbor_indexes::ElementGridConvolutionNeighborIdxs] documentation for more information
#[derive(Debug, Clone, Copy)]
pub enum ConvolutionIdentifier {
    /// The left and right neighbors
    LR(LeftRightNeighborIdentifier),
    /// The top neighbors
    Top(TopNeighborIdentifier),
    /// The bottom neighbors
    Bottom(BottomNeighborIdentifier),
    /// The center
    Center,
}

/// Identifies a coordinate on an element grid, and then uniquely identifies the chunk in the convolution that it is in
#[derive(Debug, Clone, Copy)]
pub struct ConvolutionIdx(pub JkVector, pub ConvolutionIdentifier);

/// Identifies a chunk is the left or right neighbor in the convolution
/// Check out the [super::neighbor_grids::LeftRightNeighborGrids] and
/// [super::neighbor_indexes::LeftRightNeighborIdxs] documentation for more information
#[derive(Debug, Clone, Copy)]
pub enum LeftRightNeighborIdentifier {
    /// The left neighbor
    Left,
    /// The right neighbor
    Right,
}

/// Identifies a chunk is one of the top neighbors in the convolution
/// Check out the [super::neighbor_grids::TopNeighborGrids] and
/// [super::neighbor_indexes::TopNeighborIdxs] documentation for more information
#[derive(Debug, Clone, Copy, EnumIter)]
pub enum TopNeighborIdentifier {
    /// Indicates the top neighbors are not part of a chunk doubling layer transition
    Normal(TopNeighborIdentifierNormal),
    /// Indicates a **chunk doubling** layer transition
    ChunkDoubling(TopNeighborIdentifierChunkDoubling),
}

/// Identifies that the bottom neighbor is not part of a chunk doubling layer transition
/// However, remember that it still may have half the number of cells tangentially
#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum TopNeighborIdentifierNormal {
    /// The top left neighbor
    TopLeft,
    /// The top neighbor
    #[default]
    Top,
    /// The top right neighbor
    TopRight,
}

/// Identifies a **chunk doubling** layer transition
#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum TopNeighborIdentifierChunkDoubling {
    /// The top left neighbor
    TopLeft,
    /// Second top center element, left of center
    Top1,
    /// Second top center element, right of center
    #[default]
    Top0,
    /// The top right neighbor
    TopRight,
}

/// Identifies a chunk is one of the bottom neighbors in the convolution
/// Check out the [super::neighbor_grids::BottomNeighborGrids] and
/// [super::neighbor_indexes::BottomNeighborIdxs] documentation for more information
#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BottomNeighborIdentifier {
    Normal(BottomNeighborIdentifierNormal),
    /// Indicates a **chunk doubling** layer transition
    /// In this case the chunks half because you are going down
    ChunkDoubling(BottomNeighborIdentifierChunkDoubling),
}

/// Identifies that the bottom neighbor is not part of a chunk doubling layer transition
/// However, remember that it still may have half the number of cells tangentially
#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum BottomNeighborIdentifierNormal {
    /// The bottom left neighbor
    BottomLeft,
    /// The bottom neighbor
    #[default]
    Bottom,
    /// The bottom right neighbor
    BottomRight,
}

/// Identifies a **chunk doubling** layer transition
/// (or halfing in this case because you are going down)
/// One of these will be directly below you, and be bigger than you off to one direction
/// Whereas the other will be diagonally below you
/// This depends on if your [ChunkIjkVector] has a `k` value which is even or odd
/// If it is even, then the `bl` will be directly below you, and you will be straddling its right side
/// If it is odd, then the `br` will be directly below you, and you will be straddling its left side
#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum BottomNeighborIdentifierChunkDoubling {
    /// The bottom left neighbor
    #[default]
    BottomLeft,
    /// The bottom right neighbor
    BottomRight,
}
