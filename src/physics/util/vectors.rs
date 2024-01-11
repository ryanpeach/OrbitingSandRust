use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use bevy_ecs::component::Component;
use ggez::glam::Vec2;

/// An absolute position in the world. Usually the location of some object.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct WorldCoord(pub Vec2);

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct ScreenCoord(pub Vec2);

/// A world coord vector that is relative to some position in pixel space.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RelXyPoint(pub Vec2);

impl RelXyPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2{x, y})
    }
}

impl Display for RelXyPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(RelXyPoint: ({}, {}))", self.0.x, self.0.y)
    }
}

impl Sub for RelXyPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add for RelXyPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
