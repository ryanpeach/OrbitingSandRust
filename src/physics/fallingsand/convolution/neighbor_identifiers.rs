use strum_macros::EnumIter;

use crate::physics::fallingsand::util::vectors::JkVector;

#[derive(Debug, Clone, Copy)]
pub enum LeftRightNeighborIdentifierLR {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum LeftRightNeighborIdentifier {
    LR(LeftRightNeighborIdentifierLR),
}

#[derive(Debug, Clone, Copy)]
pub enum TopNeighborIdentifierNormal {
    TopLeft,
    Top,
    TopRight,
}

#[derive(Debug, Clone, Copy)]
pub enum TopNeighborIdentifierLayerTransition {
    TopLeft,
    Top1,
    Top0,
    TopRight,
}

#[derive(Debug, Clone, Copy)]
pub enum TopNeighborIdentifier {
    Normal(TopNeighborIdentifierNormal),
    LayerTransition(TopNeighborIdentifierLayerTransition),
    SingleChunkLayerAbove,
    MultiChunkLayerAbove(usize),
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

#[derive(Debug, Clone, Copy, Default, EnumIter)]
pub enum BottomNeighborIdentifier {
    Normal(BottomNeighborIdentifierNormal),
    LayerTransition(BottomNeighborIdentifierLayerTransition),
    #[default]
    FullLayerBelow,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionIdentifier {
    LeftRight(LeftRightNeighborIdentifier),
    Top(TopNeighborIdentifier),
    Bottom(BottomNeighborIdentifier),
    Center,
}

/// The main type exported by this module
/// Identifies a coordinate on an element grid, and then uniquely identifies the chunk in the convolution that it is in
/// This is better than a hashmap because by using enums it can be quite a bit faster and more rhobust
#[derive(Debug, Clone, Copy)]
pub struct ConvolutionIdx(pub JkVector, pub ConvolutionIdentifier);
