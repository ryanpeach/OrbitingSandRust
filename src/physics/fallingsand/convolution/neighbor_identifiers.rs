use strum_macros::EnumIter;

use crate::physics::fallingsand::util::vectors::JkVector;

#[derive(Debug, Clone, Copy)]
pub enum LeftRightNeighborIdentifier {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum TopNeighborIdentifierNormal {
    TopLeft,
    #[default]
    Top,
    TopRight,
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum TopNeighborIdentifierLayerTransition {
    TopLeft,
    Top1,
    #[default]
    Top0,
    TopRight,
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum TopNeighborIdentifier {
    Normal(TopNeighborIdentifierNormal),
    ChunkDoubling(TopNeighborIdentifierLayerTransition),
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum BottomNeighborIdentifierNormal {
    BottomLeft,
    #[default]
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum BottomNeighborIdentifierLayerTransition {
    #[default]
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BottomNeighborIdentifier {
    Normal(BottomNeighborIdentifierNormal),
    ChunkDoubling(BottomNeighborIdentifierLayerTransition),
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionIdentifier {
    LR(LeftRightNeighborIdentifier),
    Top(TopNeighborIdentifier),
    Bottom(BottomNeighborIdentifier),
    Center,
}

/// The main type exported by this module
/// Identifies a coordinate on an element grid, and then uniquely identifies the chunk in the convolution that it is in
/// This is better than a hashmap because by using enums it can be quite a bit faster and more rhobust
#[derive(Debug, Clone, Copy)]
pub struct ConvolutionIdx(pub JkVector, pub ConvolutionIdentifier);
