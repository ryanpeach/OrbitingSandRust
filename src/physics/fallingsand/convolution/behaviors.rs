//! # Element Behavior API
//!
//! This module contains the behaviors of the convolution
//! It exports the [ElementGridConvolutionNeighbors] struct which contains
//! both the indexes and the grids of the neighbors
//! It contains getters for the chunks based on index or identifier
//! and both getters and setters by element
//! It also allows the user to get the element "below" or "left"/"right" of a given element
//! This is its main purpose, in defining element behaviors, being able to check
//! and change the state of the elements around it. This is the main way that the
//! elements interact with each other. Within the same chunk this is very easy,
//! but between chunks the behavior is very complex:
//!
//! 1. What does "down" mean if the resolution of cells below you is half the resolution of cells in your chunk?
//! 2. What does "up" mean if the resolution of cells above you is double the resolution of cells in your chunk?
//! 3. What if you want to go diagonally between chunks
//! 4. What are the boundaries of the chunks? Have you accessed a value you don't have ownership of?
//!
//! All of that is handled automagically by this API.

use hashbrown::HashMap;

use crate::physics::{
    fallingsand::{
        data::element_grid::ElementGrid,
        elements::element::Element,
        mesh::coordinate_directory::CoordinateDir,
        util::{
            functions::modulo,
            vectors::{ChunkIjkVector, JkVector},
        },
    },
    util::clock::Clock,
};

use super::{
    neighbor_grids::{
        BottomNeighborGrids, ConvOutOfBoundsError, ElementGridConvolutionNeighborGrids,
        LeftRightNeighborGrids, TopNeighborGrids,
    },
    neighbor_identifiers::{
        BottomNeighborIdentifier, BottomNeighborIdentifierChunkDoubling,
        BottomNeighborIdentifierNormal, ConvolutionIdentifier, ConvolutionIdx,
        LeftRightNeighborIdentifier, TopNeighborIdentifier, TopNeighborIdentifierChunkDoubling,
        TopNeighborIdentifierNormal,
    },
    neighbor_indexes::{
        BottomNeighborIdxs, ElementGridConvolutionNeighborIdxs,
        ElementGridConvolutionNeighborIdxsIter,
    },
};

/// This is the main struct exported by this module
/// It contains all the neighbors of a chunk in the convolution
/// It does not contain the target chunk itself, as this usually violates several
/// borrow checker rules
pub struct ElementGridConvolutionNeighbors {
    /// The indexes of the neighbors, tells you where they come from in the [crate::physics::fallingsand::data::element_directory::ElementGridDir]
    pub chunk_idxs: ElementGridConvolutionNeighborIdxs,
    /// The grids of the neighbors, actually stores the data
    pub grids: ElementGridConvolutionNeighborGrids,
}

/// Instantiation
impl ElementGridConvolutionNeighbors {
    /// Create a new ElementGridConvolutionNeighbors
    pub fn new(
        chunk_idxs: ElementGridConvolutionNeighborIdxs,
        mut grids: HashMap<ChunkIjkVector, ElementGrid>,
    ) -> Self {
        let lr_neighbors = LeftRightNeighborGrids::from_hashmap(&chunk_idxs.left_right, &mut grids);
        let top_neighbors = TopNeighborGrids::from_hashmap(&chunk_idxs.top, &mut grids);
        let bottom_neighbors = BottomNeighborGrids::from_hashmap(&chunk_idxs.bottom, &mut grids);
        ElementGridConvolutionNeighbors {
            chunk_idxs,
            grids: ElementGridConvolutionNeighborGrids {
                left_right: lr_neighbors,
                top: top_neighbors,
                bottom: bottom_neighbors,
            },
        }
    }

    /// Get the number of chunks
    pub fn len(&self) -> usize {
        self.chunk_idxs.iter().count()
    }
    /// Checks if there are no chunks
    pub fn is_empty(&self) -> bool {
        self.chunk_idxs.iter().count() == 0
    }
}

/// Iteration
/// We are going to implement into interation on the Neighbors so that unpackaging is easier
/// To do this we will use the into_hashmap method on the neighbor grids
/// and the iter method on the neighbor indexes
/// taking from the hashmap on each iteration of the iter
pub struct ElementGridConvolutionNeighborsIntoIter {
    /// The iterator for the neighbor indexes
    chunk_idxs_iter: ElementGridConvolutionNeighborIdxsIter,
    /// The grids of the neighbors
    grids: HashMap<ChunkIjkVector, ElementGrid>,
}

impl Iterator for ElementGridConvolutionNeighborsIntoIter {
    type Item = (ChunkIjkVector, ElementGrid);
    fn next(&mut self) -> Option<Self::Item> {
        match self.chunk_idxs_iter.next() {
            Some(chunk_idx) => {
                let grid = self.grids.remove(&chunk_idx)?;
                Some((chunk_idx, grid))
            }
            None => None,
        }
    }
}

impl IntoIterator for ElementGridConvolutionNeighbors {
    type Item = (ChunkIjkVector, ElementGrid);
    type IntoIter = ElementGridConvolutionNeighborsIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        ElementGridConvolutionNeighborsIntoIter {
            chunk_idxs_iter: self.chunk_idxs.iter(),
            grids: self.grids.into_hashmap(),
        }
    }
}

/// Coordinate tranformations
/// Methods which allow you to get an index "relative" to another index on the element convolution.
impl ElementGridConvolutionNeighbors {
    /// Gets the element n cells below the position in the target chunk
    /// the target chunk is the chunk in the center of the convolution
    /// the pos is the position in the target chunk
    pub fn get_below_idx_from_center(
        &self,
        target_chunk: &ElementGrid,
        _coord_dir: &CoordinateDir,
        pos: &JkVector,
        n: usize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        // Handle naive case where you don't change your chunk
        if pos.j >= n {
            return Ok(ConvolutionIdx(
                JkVector::new(pos.j - n, pos.k),
                ConvolutionIdentifier::Center,
            ));
        }

        // Handle error cases where you go beyond the chunk below you
        let b_concentric_circles = self.grids.bottom.get_num_concentric_circles();
        if (pos.j as isize - n as isize + b_concentric_circles as isize) < 0 {
            return Err(ConvOutOfBoundsError(ConvolutionIdx(
                JkVector { j: pos.j, k: pos.k },
                ConvolutionIdentifier::Center,
            )));
        }

        match self.chunk_idxs.bottom {
            // If there is no layer below you, error out
            BottomNeighborIdxs::BottomOfGrid => Err(ConvOutOfBoundsError(ConvolutionIdx(
                JkVector { j: pos.j, k: pos.k },
                ConvolutionIdentifier::Center,
            ))),
            // TODO: Unit test
            BottomNeighborIdxs::ChunkDoubling { .. } => {
                let mut new_coords = JkVector {
                    j: pos.j + b_concentric_circles - n,
                    k: pos.k / 2,
                };
                // If you are an even index chunk, the right half of bl is the same as b
                // If you are on odd index chunk, the left half of br is the same as b
                // TODO: document this with pictures
                // TODO: Unit test
                let transition = if target_chunk.get_chunk_coords().get_chunk_idx().k % 2 == 0 {
                    BottomNeighborIdentifierChunkDoubling::BottomLeft
                } else {
                    new_coords.k += self.grids.bottom.get_num_radial_lines() / 2;
                    BottomNeighborIdentifierChunkDoubling::BottomRight
                };
                Ok(ConvolutionIdx(
                    new_coords,
                    ConvolutionIdentifier::Bottom(BottomNeighborIdentifier::ChunkDoubling(
                        transition,
                    )),
                ))
            }
            BottomNeighborIdxs::Normal { .. } => {
                let mut new_coords = JkVector {
                    j: pos.j + b_concentric_circles - n,
                    k: pos.k,
                };
                // Sometimes a "Normal" bottom index is actually on a different layer
                // just with the same number of tangential chunkss
                // If there are the same number of radial lines (not a layer transition) we dont
                // need to divide k by 2
                let this_radial_lines = target_chunk.get_chunk_coords().get_num_radial_lines();
                let b_radial_lines = self.grids.bottom.get_num_radial_lines();
                if this_radial_lines != b_radial_lines {
                    new_coords.k = pos.k / 2;
                }
                Ok(ConvolutionIdx(
                    new_coords,
                    ConvolutionIdentifier::Bottom(BottomNeighborIdentifier::Normal(
                        BottomNeighborIdentifierNormal::Bottom,
                    )),
                ))
            }
        }
    }

    /// Positive k is left, counter clockwise
    /// Negative k is right, clockwise
    pub fn get_left_right_idx_from_center(
        &self,
        target_chunk: &ElementGrid,
        pos: &JkVector,
        rk: isize,
    ) -> Result<ConvolutionIdx, ConvOutOfBoundsError> {
        // In the left right direction, unlike up down, every chunk has the same number of radial lines
        let radial_lines = target_chunk.get_chunk_coords().get_num_radial_lines();

        // You should not be doing any loops that might make you re-target yourself
        if rk.abs() >= radial_lines as isize {
            return Err(ConvOutOfBoundsError(ConvolutionIdx(
                JkVector {
                    j: pos.j,
                    k: modulo(pos.k as isize + rk, radial_lines),
                },
                ConvolutionIdentifier::Center,
            )));
        }

        let new_k = modulo(pos.k as isize + rk, radial_lines);

        if pos.k as isize + rk >= radial_lines as isize {
            Ok(ConvolutionIdx(
                JkVector { j: pos.j, k: new_k },
                ConvolutionIdentifier::LR(LeftRightNeighborIdentifier::Left),
            ))
        } else if pos.k as isize + rk < 0 {
            Ok(ConvolutionIdx(
                JkVector { j: pos.j, k: new_k },
                ConvolutionIdentifier::LR(LeftRightNeighborIdentifier::Right),
            ))
        } else {
            Ok(ConvolutionIdx(
                JkVector { j: pos.j, k: new_k },
                ConvolutionIdentifier::Center,
            ))
        }
    }
}

/// Errors for the getter methods
#[derive(Debug)]
pub enum GetChunkErr {
    /// You aren't allowed to get the center chunk.
    /// As [ElementGridConvolutionNeighbors] describes in its documentation,
    /// it doesn't contain the center chunk.
    CenterChunk,
}

/// Getter and setter methods
impl ElementGridConvolutionNeighbors {
    /// TODO, use macros to get rid of redundancy
    fn get_chunk_mut(
        &mut self,
        id: ConvolutionIdentifier,
    ) -> Result<&mut ElementGrid, GetChunkErr> {
        match id {
            ConvolutionIdentifier::Top(top_id) => match top_id {
                TopNeighborIdentifier::Normal(normal_id) => match normal_id {
                    TopNeighborIdentifierNormal::Top { .. } => {
                        if let TopNeighborGrids::Normal { t, .. } = &mut self.grids.top {
                            Ok(t)
                        } else {
                            panic!("Tried to get t chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopLeft { .. } => {
                        if let TopNeighborGrids::Normal { tl, .. } = &mut self.grids.top {
                            Ok(tl)
                        } else {
                            panic!("Tried to get tl chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopRight { .. } => {
                        if let TopNeighborGrids::Normal { tr, .. } = &mut self.grids.top {
                            Ok(tr)
                        } else {
                            panic!("Tried to get tr chunk that doesn't exist")
                        }
                    }
                },
                TopNeighborIdentifier::ChunkDoubling(layer_transition_id) => {
                    match layer_transition_id {
                        TopNeighborIdentifierChunkDoubling::Top0 { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { t0, .. } = &mut self.grids.top
                            {
                                Ok(t0)
                            } else {
                                panic!("Tried to get t0 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::Top1 { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { t1, .. } = &mut self.grids.top
                            {
                                Ok(t1)
                            } else {
                                panic!("Tried to get t1 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::TopLeft { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { tl, .. } = &mut self.grids.top
                            {
                                Ok(tl)
                            } else {
                                panic!("Tried to get tl chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::TopRight { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { tr, .. } = &mut self.grids.top
                            {
                                Ok(tr)
                            } else {
                                panic!("Tried to get tr chunk that doesn't exist")
                            }
                        }
                    }
                }
            },
            ConvolutionIdentifier::Bottom(bottom_id) => match bottom_id {
                BottomNeighborIdentifier::Normal(normal_id) => match normal_id {
                    BottomNeighborIdentifierNormal::Bottom { .. } => {
                        if let BottomNeighborGrids::Normal { b, .. } = &mut self.grids.bottom {
                            Ok(b)
                        } else {
                            panic!("Tried to get b chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomLeft { .. } => {
                        if let BottomNeighborGrids::Normal { bl, .. } = &mut self.grids.bottom {
                            Ok(bl)
                        } else {
                            panic!("Tried to get bl chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomRight { .. } => {
                        if let BottomNeighborGrids::Normal { br, .. } = &mut self.grids.bottom {
                            Ok(br)
                        } else {
                            panic!("Tried to get br chunk that doesn't exist")
                        }
                    }
                },
                BottomNeighborIdentifier::ChunkDoubling(layer_transition_id) => {
                    match layer_transition_id {
                        BottomNeighborIdentifierChunkDoubling::BottomLeft { .. } => {
                            if let BottomNeighborGrids::ChunkDoubling { bl, .. } =
                                &mut self.grids.bottom
                            {
                                Ok(bl)
                            } else {
                                panic!("Tried to get bl chunk that doesn't exist")
                            }
                        }
                        BottomNeighborIdentifierChunkDoubling::BottomRight { .. } => {
                            if let BottomNeighborGrids::ChunkDoubling { br, .. } =
                                &mut self.grids.bottom
                            {
                                Ok(br)
                            } else {
                                panic!("Tried to get br chunk that doesn't exist")
                            }
                        }
                    }
                }
            },
            ConvolutionIdentifier::LR(lr_id) => match lr_id {
                LeftRightNeighborIdentifier::Left { .. } => {
                    let LeftRightNeighborGrids::LR { l, .. } = &mut self.grids.left_right;
                    Ok(l)
                }
                LeftRightNeighborIdentifier::Right { .. } => {
                    let LeftRightNeighborGrids::LR { r, .. } = &mut self.grids.left_right;
                    Ok(r)
                }
            },
            ConvolutionIdentifier::Center => Err(GetChunkErr::CenterChunk),
        }
    }

    /// Get the chunk identified by the given identifier
    fn get_chunk(&self, id: ConvolutionIdentifier) -> Result<&ElementGrid, GetChunkErr> {
        match id {
            ConvolutionIdentifier::Top(top_id) => match top_id {
                TopNeighborIdentifier::Normal(normal_id) => match normal_id {
                    TopNeighborIdentifierNormal::Top { .. } => {
                        if let TopNeighborGrids::Normal { t, .. } = &self.grids.top {
                            Ok(t)
                        } else {
                            panic!("Tried to get t chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopLeft { .. } => {
                        if let TopNeighborGrids::Normal { tl, .. } = &self.grids.top {
                            Ok(tl)
                        } else {
                            panic!("Tried to get tl chunk that doesn't exist")
                        }
                    }
                    TopNeighborIdentifierNormal::TopRight { .. } => {
                        if let TopNeighborGrids::Normal { tr, .. } = &self.grids.top {
                            Ok(tr)
                        } else {
                            panic!("Tried to get tr chunk that doesn't exist")
                        }
                    }
                },
                TopNeighborIdentifier::ChunkDoubling(layer_transition_id) => {
                    match layer_transition_id {
                        TopNeighborIdentifierChunkDoubling::Top0 { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { t0, .. } = &self.grids.top {
                                Ok(t0)
                            } else {
                                panic!("Tried to get t0 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::Top1 { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { t1, .. } = &self.grids.top {
                                Ok(t1)
                            } else {
                                panic!("Tried to get t1 chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::TopLeft { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { tl, .. } = &self.grids.top {
                                Ok(tl)
                            } else {
                                panic!("Tried to get tl chunk that doesn't exist")
                            }
                        }
                        TopNeighborIdentifierChunkDoubling::TopRight { .. } => {
                            if let TopNeighborGrids::ChunkDoubling { tr, .. } = &self.grids.top {
                                Ok(tr)
                            } else {
                                panic!("Tried to get tr chunk that doesn't exist")
                            }
                        }
                    }
                }
            },
            ConvolutionIdentifier::Bottom(bottom_id) => match bottom_id {
                BottomNeighborIdentifier::Normal(normal_id) => match normal_id {
                    BottomNeighborIdentifierNormal::Bottom { .. } => {
                        if let BottomNeighborGrids::Normal { b, .. } = &self.grids.bottom {
                            Ok(b)
                        } else {
                            panic!("Tried to get b chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomLeft { .. } => {
                        if let BottomNeighborGrids::Normal { bl, .. } = &self.grids.bottom {
                            Ok(bl)
                        } else {
                            panic!("Tried to get bl chunk that doesn't exist")
                        }
                    }
                    BottomNeighborIdentifierNormal::BottomRight { .. } => {
                        if let BottomNeighborGrids::Normal { br, .. } = &self.grids.bottom {
                            Ok(br)
                        } else {
                            panic!("Tried to get br chunk that doesn't exist")
                        }
                    }
                },
                BottomNeighborIdentifier::ChunkDoubling(layer_transition_id) => {
                    match layer_transition_id {
                        BottomNeighborIdentifierChunkDoubling::BottomLeft { .. } => {
                            if let BottomNeighborGrids::ChunkDoubling { bl, .. } =
                                &self.grids.bottom
                            {
                                Ok(bl)
                            } else {
                                panic!("Tried to get bl chunk that doesn't exist")
                            }
                        }
                        BottomNeighborIdentifierChunkDoubling::BottomRight { .. } => {
                            if let BottomNeighborGrids::ChunkDoubling { br, .. } =
                                &self.grids.bottom
                            {
                                Ok(br)
                            } else {
                                panic!("Tried to get br chunk that doesn't exist")
                            }
                        }
                    }
                }
            },
            ConvolutionIdentifier::LR(lr_id) => match lr_id {
                LeftRightNeighborIdentifier::Left { .. } => {
                    let LeftRightNeighborGrids::LR { l, .. } = &self.grids.left_right;
                    Ok(l)
                }
                LeftRightNeighborIdentifier::Right { .. } => {
                    let LeftRightNeighborGrids::LR { r, .. } = &self.grids.left_right;
                    Ok(r)
                }
            },
            ConvolutionIdentifier::Center => Err(GetChunkErr::CenterChunk),
        }
    }

    /// Get the element at the given index
    pub fn get(
        &self,
        target_grid: &ElementGrid,
        idx: ConvolutionIdx,
    ) -> Result<Box<dyn Element>, ConvOutOfBoundsError> {
        match idx.1 {
            ConvolutionIdentifier::Center => match target_grid.checked_get(idx.0) {
                Ok(element) => Ok(element.box_clone()),
                Err(_) => Err(ConvOutOfBoundsError(idx)),
            },
            _ => match self.get_chunk(idx.1) {
                Ok(chunk) => match chunk.checked_get(idx.0) {
                    Ok(element) => Ok(element.box_clone()),
                    Err(_) => Err(ConvOutOfBoundsError(idx)),
                },
                Err(GetChunkErr::CenterChunk) => {
                    unreachable!("This should never happen because we are checking for it in the match idx.1 statement")
                }
            },
        }
    }

    /// Replace the element at the given index
    /// Great for taking ownership of the element
    /// Can also be used to give ownership back
    /// Returns an error if the index is out of bounds
    pub fn replace(
        &mut self,
        target_grid: &mut ElementGrid,
        idx: ConvolutionIdx,
        element: Box<dyn Element>,
        current_time: Clock,
    ) -> Result<Box<dyn Element>, ConvOutOfBoundsError> {
        match idx.1 {
            ConvolutionIdentifier::Center => {
                let out = target_grid.replace(idx.0, element, current_time);
                Ok(out)
            }
            _ => match self.get_chunk_mut(idx.1) {
                Ok(chunk) => {
                    let out = chunk.replace(idx.0, element, current_time);
                    Ok(out)
                }
                Err(GetChunkErr::CenterChunk) => {
                    unreachable!("This should never happen because we are checking for it in the match idx.1 statement")
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::fallingsand::{
        data::element_directory::ElementGridDir, mesh::coordinate_directory::CoordinateDirBuilder,
    };

    mod get_below_idx_from_center {
        use super::*;
        use crate::physics::{fallingsand::util::vectors::IjkVector, orbits::components::Length};

        /// The default element grid directory for testing
        fn get_element_grid_dir() -> ElementGridDir {
            let coordinate_dir = CoordinateDirBuilder::new()
                .cell_radius(Length(1.0))
                .num_layers(10)
                .first_num_radial_lines(6)
                .second_num_concentric_circles(3)
                .max_concentric_circles_per_chunk(128)
                .max_radial_lines_per_chunk(128)
                .build();
            ElementGridDir::new_empty(coordinate_dir)
        }

        fn _test_get_below_idx_from_center(pos1: IjkVector, pos2: IjkVector) {
            let mut element_dir = get_element_grid_dir();
            let chunk_pos1 = element_dir.get_coordinate_dir().cell_idx_to_chunk_idx(pos1);
            let chunk_pos2 = element_dir.get_coordinate_dir().cell_idx_to_chunk_idx(pos2);
            let mut package = element_dir
                .package_coordinate_neighbors(chunk_pos1.0)
                .unwrap();
            let chunk = element_dir.get_chunk_by_chunk_ijk(chunk_pos1.0);
            let should_eq_pos2 = package
                .get_below_idx_from_center(
                    chunk,
                    element_dir.get_coordinate_dir(),
                    &chunk_pos1.1,
                    1,
                )
                .unwrap();
            assert_eq!(chunk_pos2.1, should_eq_pos2.0, "The position is incorrect");

            // Check that the get_chunk method also works
            let should_eq_chunk2 = match package.get_chunk(should_eq_pos2.1) {
                Ok(chunk) => chunk.get_chunk_coords().get_chunk_idx(),
                Err(GetChunkErr::CenterChunk) => chunk_pos2.0,
            };
            // Test the mut version too
            let should_eq_chunk2_mut = match package.get_chunk_mut(should_eq_pos2.1) {
                Ok(chunk) => chunk.get_chunk_coords().get_chunk_idx(),
                Err(GetChunkErr::CenterChunk) => chunk_pos2.0,
            };
            assert_eq!(chunk_pos2.0, should_eq_chunk2, "get_chunk is not working");
            assert_eq!(
                chunk_pos2.0, should_eq_chunk2_mut,
                "get_chunk_mut is not working"
            );
        }

        macro_rules! test_get_below_idx_from_center {
            ($name:ident, $pos1:expr, $pos2:expr) => {
                #[test]
                fn $name() {
                    _test_get_below_idx_from_center(
                        IjkVector::new($pos1.0, $pos1.1, $pos1.2),
                        IjkVector::new($pos2.0, $pos2.1, $pos2.2),
                    );
                }
            };
        }

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i2_j2_k1,
            (2, 2, 1),
            (2, 1, 1)
        );

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i2_j0_k8,
            (2, 0, 8),
            (1, 2, 4)
        );

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i3_j0_k10,
            (3, 0, 10),
            (2, 5, 5)
        );

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i6_j0_k180,
            (6, 0, 180),
            (5, 47, 90)
        );

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i7_j0_k355,
            (7, 0, 355),
            (6, 95, 355 / 2)
        );

        test_get_below_idx_from_center!(
            test_get_below_idx_from_center_i7_j0_k420,
            (7, 0, 420),
            (6, 95, 210)
        );
    }

    mod get_left_right_idx_from_center {
        use super::*;
        use crate::physics::{fallingsand::util::vectors::IjkVector, orbits::components::Length};

        /// The default element grid directory for testing
        fn get_element_grid_dir() -> ElementGridDir {
            let coordinate_dir = CoordinateDirBuilder::new()
                .cell_radius(Length(1.0))
                .num_layers(7)
                .first_num_radial_lines(12)
                .second_num_concentric_circles(3)
                .first_num_tangential_chunkss(3)
                .max_radial_lines_per_chunk(128)
                .max_concentric_circles_per_chunk(128)
                .build();
            ElementGridDir::new_empty(coordinate_dir)
        }

        fn _test_get_left_right_idx_from_center(pos1: IjkVector, n: isize, pos2: IjkVector) {
            let mut element_dir = get_element_grid_dir();
            let chunk_pos1 = element_dir.get_coordinate_dir().cell_idx_to_chunk_idx(pos1);
            let chunk_pos2 = element_dir.get_coordinate_dir().cell_idx_to_chunk_idx(pos2);
            let mut package = element_dir
                .package_coordinate_neighbors(chunk_pos1.0)
                .unwrap();
            let chunk = element_dir.get_chunk_by_chunk_ijk(chunk_pos1.0);
            let should_eq_pos2 = package
                .get_left_right_idx_from_center(chunk, &chunk_pos1.1, n)
                .unwrap();
            assert_eq!(chunk_pos2.1, should_eq_pos2.0, "The position is incorrect");

            // Check that the get_chunk method also works
            let should_eq_chunk2 = match package.get_chunk(should_eq_pos2.1) {
                Ok(chunk) => chunk.get_chunk_coords().get_chunk_idx(),
                Err(GetChunkErr::CenterChunk) => chunk_pos2.0,
            };
            // Test the mut version too
            let should_eq_chunk2_mut = match package.get_chunk_mut(should_eq_pos2.1) {
                Ok(chunk) => chunk.get_chunk_coords().get_chunk_idx(),
                Err(GetChunkErr::CenterChunk) => chunk_pos2.0,
            };
            assert_eq!(chunk_pos2.0, should_eq_chunk2, "get_chunk is not working");
            assert_eq!(
                chunk_pos2.0, should_eq_chunk2_mut,
                "get_chunk_mut is not working"
            );
        }

        macro_rules! test_get_left_right_idx_from_center {
            ($name:ident, $pos1:expr, $n:expr, $pos2:expr) => {
                #[test]
                fn $name() {
                    _test_get_left_right_idx_from_center(
                        IjkVector::new($pos1.0, $pos1.1, $pos1.2),
                        $n,
                        IjkVector::new($pos2.0, $pos2.1, $pos2.2),
                    )
                }
            };
        }

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k32_right,
            (2, 0, 32),
            -1,
            (2, 0, 31)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k31_left,
            (2, 0, 31),
            1,
            (2, 0, 32)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k47_left,
            (2, 0, 47),
            1,
            (2, 0, 0)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k0_right,
            (2, 0, 0),
            -1,
            (2, 0, 47)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k1_left,
            (2, 0, 1),
            1,
            (2, 0, 2)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i2_j0_k1_right,
            (2, 0, 1),
            -1,
            (2, 0, 0)
        );

        test_get_left_right_idx_from_center!(
            test_get_left_right_idx_from_center_i5_j21_k383_right,
            (5, 21, 0),
            -1,
            (5, 21, 383)
        );
    }
}
