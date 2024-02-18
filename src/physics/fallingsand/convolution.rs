//! The convolution module contains the logic for getting elements from locations relative to another element
//! in a way that is convienient for parallel processing of chunks.
//!
//! # Chunk Convolution and the Borrow Checker
//!
//! Chunk convolutions are the basis for the speed of all the planetary simulations in the game
//! The idea is that we iterate over all the chunks in a
//! [super::data::element_directory::ElementGridDir] and for each of them package
//! all their neighbors into a single
//! struct, called the [self::neighbor_grids::ElementGridConvolutionNeighborGrids].
//! We move these neighbor chunks out of a collection using `take`
//! so that we have complete ownership of them. Then we process the chunk with its
//! neighbors as context, and then put them back into the collection.
//! We do this all in parallel using rayon.
//!
//! Because every neighbor is packaged with the chunk we are processing, we have to
//! convolve over the grid in steps of 4.
//! Because in an array like:
//!
//! ```text
//! 0 1 2 3
//! ^     ^
//! ```
//!
//! 0 has 1 as its right neighbor, 3 has 2 as its left neighbor, so we can only in
//! parallel process 0 and 3 together.
//!
//! > **TIP**
//! > Please familiarize yourself with [super::mesh::chunk_coords] and
//! > [super::mesh::coordinate_directory] documentation before continuing to understand
//! > chunk layouts etc.
//!
//! # The Problem
//!
//! The problem is that between layers of the world, we have unusual numbers of chunks
//! either to the bottom or to the top of us. In the middle of a layer its easy! But
//! between layers which are not the same number of chunks, there is an order of 2
//! difference that layers number of chunks and out own.
//!
//! In this image, a "layer" can be identified by having a consistent "chunk density"
//! between layers chunks tend to double either in the radial or tangential direction.
//! Chunks may also change size between layers.
//!
//! ![default chunks](../../../../assets/docs/wireframe/default_chunks.png)
//!
//! This makes accessing a cell (a member of a chunk) "above" or "below" us difficult,
//! because we have to know if we are crossing a layer transition or not, and how to handle it.
//!
//! # The Solution
//!
//! The primary submodule exported by this module is the [self::behaviors] module.
//!
//! It gives you an API to get elements from locations relative to another element in
//! a convolution.
//!
//! This greatly simplifies the code in the [super::elements] module, otherwise this
//! game would basically be impossible to be chunked.
//!
//! Please continue by reading the documentation for the [self::behaviors] module.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod behaviors;
pub mod neighbor_grids;
pub mod neighbor_identifiers;
pub mod neighbor_indexes;
