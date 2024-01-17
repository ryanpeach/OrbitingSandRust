//! Vectors can be used to represent many things
//! This module contains all the vector types used in the game.

use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use bevy::{math::Vec2, render::color::Color};

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

/// A vertex in a mesh
/// Originally from ggez
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

/// A rectangle
/// Originally from ggez
/// TODO: Replace with bevy::math::Rect
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<Rect> for bevy::math::Rect {
    fn from(val: Rect) -> Self {
        bevy::math::Rect::new(val.x, val.y, val.x + val.w, val.y + val.h)
    }
}

impl From<bevy::math::Rect> for Rect {
    fn from(rect: bevy::math::Rect) -> Self {
        Self {
            x: rect.min.x,
            y: rect.min.y,
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
