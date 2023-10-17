use std::fmt::Display;

use ggez::glam::Vec2;

/// A world coord vector that is relative to some position in pixel space
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RelXyPoint(pub Vec2);

impl Display for RelXyPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(RelXyPoint: ({}, {}))", self.0.x, self.0.y)
    }
}
