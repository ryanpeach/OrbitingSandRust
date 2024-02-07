//! This module defines the "coordinates" for the chunk system.
//! This module does not store data, is copyable, etc. It mostly handles the math
//! for drawing the radial coordinates of the chunk system.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/// Defines the coordinates of a chunk itself.
pub mod chunk_coords;

/// A directory of chunks forming a full coordinate system.
pub mod coordinate_directory;
