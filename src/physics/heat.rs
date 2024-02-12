//! Heat related physics.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/// Most of the units of measure as well as bevy components for heat related physics.
pub mod components;

/// Heat related math functions such as the heat equation.
pub mod math;

/// Heat also needs to be chunk convoluted, but the implementation is a little simpler than [crate::physics::fallingsand::convolution].
pub mod convolution;
