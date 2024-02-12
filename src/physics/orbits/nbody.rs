//! # NxM-Body Physics
//!
//! In this module, we implement a type of gravitational physics simulation I will call
//! "NxM-Body" physics.
//!
//! Similar to N-Body physics, this models a more complex system of gravity than mere
//! two-body physics. However, it is not as complex as N-Body physics, which models the
//! gravitational interactions between all bodies in the system. Instead, NxM-Body physics
//! models the gravitational interactions between all bodies in the system that have a
//! gravitational field, which will likely just be the planets, moons, and stars, likely
//! less than 100 bodies. We can call this N. The other bodies, which will likely be
//! spacecraft, asteroids, and other small bodies, will be modeled as if they are affected
//! by the gravitational fields of the bodies with gravitational fields, but not by each
//! other. We can call this M.
//!
//! This is a simple model that is much less computationally expensive than N-Body physics,
//! which is worst-case O(N^2), wheras NxM-Body physics is worst-case O(N*M). Since M is
//! constant and small in any given simulation, this is effectively O(N).
//!
//! # Leapfrog Integration
//!
//! <https://en.wikipedia.org/wiki/Leapfrog_integration>
//!
//! We use the leapfrog integration method to update the positions and velocities of the
//! bodies. This is a [symplectic integrator](https://en.wikipedia.org/wiki/Symplectic_integrator)
//! , which means it conserves energy. This is
//! important for a physics simulation, because it means that the simulation will not
//! "drift" over time, and the bodies will not gain or lose energy over time.
//!
//! It is also a second-order integrator, which means that it is more accurate than a
//! first-order integrator, like Euler's method. This means that the simulation will be
//! more accurate, and will not require as small of a time step to be stable.
//!
//! It is acomplished by updating the velocity of the bodies by half a time step, then
//! updating the position of the bodies by a full time step, then updating the velocity
//! of the bodies by half a time step again.
//!
//! # Decision to use the CPU
//!
//! I originally wanted to use the GPU for this simulation, because it is a very parallel
//! problem. However, I decided to use the CPU because it is a simpler solution, and each
//! frame I would need to read the positions and velocities of all the bodies from the GPU
//! which is actually slower than just doing the computation on the CPU. Using rayon, I can
//! parallelize the computation on the CPU, so it is not a huge performance hit.

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        entity::Entity,
        query::{With, Without},
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    math::{Vec2, Vec3Swizzles},
    time::Time,
    transform::components::Transform,
};

use super::components::{ForceVec, GravitationalField, Mass, Velocity};

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
/// This is squared because then we don't have to do a square root to compare it to the distance squared in the force calculation.
pub const MIN_DISTANCE_SQUARED: f32 = 100.0;

/// The gravitational constant
///
/// $ \frac{N \cdot m^2}{kg^2} $
///
/// Note this has nothing to do with real life, and is just a scaling
/// factor to make the simulation
/// act at the scale of gravity we want.
pub const G: f32 = 1.0e3;

/// Returns the gravitational force between two entities
fn compute_gravitational_force(
    pos1: &Transform,
    mass1: &Mass,
    pos2: &Transform,
    mass2: &Mass,
) -> ForceVec {
    let r = pos2.translation - pos1.translation;
    let mut distance_squared = r.length_squared();
    distance_squared = distance_squared.max(MIN_DISTANCE_SQUARED);

    // The gravitational constant G and masses are factored into the force magnitude
    let force_magnitude = G * mass1.0 * mass2.0 / distance_squared;

    // Calculate the force direction
    // Normalize the displacement vector (r) to get the direction
    let force_direction = r.normalize();

    // The final force vector is the direction scaled by the force magnitude
    let out = ForceVec((force_direction * force_magnitude).xy());
    assert!(out.0.is_finite(), "force: {:?}", out);
    out
}

/// Updates the velocity of the entity one half step
fn half_step_velocity_update(
    this_body: (Entity, &Transform, &mut Velocity, &Mass),
    other_bodies: &[(Entity, Transform, Velocity, Mass)],
    dt: f32,
) {
    let mut net_force = Vec2::ZERO;
    for other_body in other_bodies {
        if this_body.0 == other_body.0 {
            continue;
        }
        let force =
            compute_gravitational_force(this_body.1, this_body.3, &other_body.1, &other_body.3);
        net_force += force.0;
    }
    // If mass is 0, don't update the velocity
    if this_body.3 .0 == 0.0 {
        return;
    }
    let vdiff = net_force / this_body.3 .0 * (dt / 2.0);
    assert!(vdiff.is_finite(), "vdiff: {:?}", vdiff);
    this_body.2 .0 += vdiff;
}

/// Updates the position of the entity one full step
fn full_position_update(this_body: (Entity, &mut Transform, &Velocity, &Mass), dt: f32) {
    let pdiff = (this_body.2 .0 * dt).extend(0.0);
    assert!(pdiff.is_finite(), "pdiff: {:?}", pdiff);
    this_body.1.translation += pdiff;
}

/// Plugin to set up nbody physics
pub struct NBodyPlugin;

impl Plugin for NBodyPlugin {
    /// Adds the systems for the plugin
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::grav_bodies_system,
                Self::no_grav_bodies_system.after(Self::grav_bodies_system),
            ),
        );
    }
}

/// Systems for the plugin
impl NBodyPlugin {
    /// Updates the locations and velocities of the entities with gravitational fields
    /// based on each other entity with a gravitational field
    fn grav_bodies_system(
        mut grav_bodies: Query<
            (Entity, &mut Transform, &mut Velocity, &Mass),
            With<GravitationalField>,
        >,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        let grav_bodies_copy = grav_bodies
            .iter()
            .map(|(entity, transform, velocity, mass)| (entity, *transform, *velocity, *mass))
            .collect::<Vec<_>>();
        grav_bodies
            .par_iter_mut()
            .for_each(|(entity, mut transform, mut velocity, mass)| {
                half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
                full_position_update((entity, &mut transform, &velocity, mass), dt);
                half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
            });
    }

    /// Updates the locations and velocities of the entities without gravitational fields
    /// based on the entities with gravitational fields
    fn no_grav_bodies_system(
        mut no_grav_bodies: Query<
            (Entity, &mut Transform, &mut Velocity, &Mass),
            Without<GravitationalField>,
        >,
        grav_bodies: Query<
            (Entity, &mut Transform, &mut Velocity, &Mass),
            With<GravitationalField>,
        >,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        let grav_bodies_copy = grav_bodies
            .iter()
            .map(|(entity, transform, velocity, mass)| (entity, *transform, *velocity, *mass))
            .collect::<Vec<_>>();
        no_grav_bodies
            .par_iter_mut()
            .for_each(|(entity, mut transform, mut velocity, mass)| {
                half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
                full_position_update((entity, &mut transform, &velocity, mass), dt);
                half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
            });
    }
}
