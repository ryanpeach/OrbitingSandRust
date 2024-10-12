#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{ecs::component::Component, math::Vec2};
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

use crate::physics::orbits::nbody::GravitationalAcceleration;

/// The mass of an entity in kilograms.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign, Sum)]
pub struct Mass(pub f32);

/// The velocity of an entity in meters per second.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct Velocity(pub Vec2);

/// The force applied to an entity with its direction $\vec{N}$
#[derive(Component, Debug, Clone, Copy)]
pub struct ForceVec(pub Vec2);

/// The scalar force applied to an entity in Newtons $N$
#[derive(Component, Debug, Clone, Copy)]
pub struct Force(pub f32);

impl From<ForceVec> for Force {
    fn from(force_vec: ForceVec) -> Self {
        Force(force_vec.0.length())
    }
}

/// A length in meters.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign, Sum)]
pub struct Length(pub f32);

impl Default for Length {
    fn default() -> Self {
        Length(1.0)
    }
}

impl Length {
    /// Returns the area of the length
    #[must_use]
    pub fn area(&self) -> Area {
        Area(self.0 * self.0)
    }
}

/// An area in square meters.
#[derive(Component, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign, Sum)]
pub struct Area(pub f32);

impl Default for Area {
    fn default() -> Self {
        Area(1.0)
    }
}
