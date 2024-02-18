#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{ecs::component::Component, math::Vec2};
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

use super::nbody::G;

/// Indicates that an entity emits a gravitational field.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct GravitationalField;

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

impl Force {
    /// Returns the force applied to the entity by gravitation
    pub fn from_mass(mass: Mass, acceleration: GravitationalAcceleration) -> Self {
        Force(mass.0 * acceleration.0)
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

/// The acceleration due to gravity
#[derive(Component, Debug, Clone, Copy)]
pub struct GravitationalAcceleration(pub f32);

impl GravitationalAcceleration {
    /// Returns the acceleration due to gravity towards a mass
    pub fn from_total_mass(total_mass: Mass) -> Self {
        GravitationalAcceleration(G * total_mass.0)
    }
}
