use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use bevy::{ecs::component::Component, render::color::Color};
use glam::Vec2;
use mint::Point2;

/// An absolute position in the world. Usually the location of some object.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct WorldCoord(pub Vec2);

impl Default for WorldCoord {
    fn default() -> Self {
        Self(Vec2 { x: 0.0, y: 0.0 })
    }
}

impl Into<Point2<f32>> for WorldCoord {
    fn into(self) -> Point2<f32> {
        Point2 {
            x: self.0.x,
            y: self.0.y,
        }
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct ScreenCoord(pub Vec2);

impl Default for ScreenCoord {
    fn default() -> Self {
        Self(Vec2 { x: 0.0, y: 0.0 })
    }
}

impl Into<Point2<f32>> for ScreenCoord {
    fn into(self) -> Point2<f32> {
        Point2 {
            x: self.0.x,
            y: self.0.y,
        }
    }
}

/// A world coord vector that is relative to some position in pixel space.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RelXyPoint(pub Vec2);

impl RelXyPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2 { x, y })
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

pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Into<bevy::math::Rect> for Rect {
    fn into(self) -> bevy::math::Rect {
        bevy::math::Rect::new(self.x, self.y, self.x + self.w, self.y + self.h)
    }
}

impl From<bevy::math::Rect> for Rect {
    fn from(rect: bevy::math::Rect) -> Self {
        Self {
            x: rect.x(),
            y: rect.y(),
            w: rect.width(),
            h: rect.height(),
        }
    }
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}
