use bevy::prelude::*;

use super::components::{GravitationalField, Mass, Velocity};

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
const MIN_DISTANCE_SQUARED: f32 = 1.0;

pub fn leapfrog_integration_system(
    mut no_grav_query: Query<(&Mass, &mut Velocity, &mut Transform), Without<GravitationalField>>,
    mut grav_query: Query<(&Mass, &mut Velocity, &mut Transform), With<GravitationalField>>,
    time: Res<Time>,
) {
    debug!(
        "leapfrog_integration_system: dt: {} nb_no_grav: {} nb_grav: {}",
        time.delta_seconds(),
        no_grav_query.iter().count(),
        grav_query.iter().count(),
    );
    let dt = time.delta_seconds();
    let half_dt = dt * 0.5;

    // Handle interactions between bodies with gravity and bodies without gravity
    // First half-step velocity update
    no_grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &grav_query {
                let force =
                    compute_gravitational_force(&position, mass, other_position, other_mass);
                net_force += force;
            }

            velocity.0 += net_force / mass.0 * half_dt;
        });

    // Full-step position update
    no_grav_query
        .par_iter_mut()
        .for_each(|(_, velocity, mut position)| {
            let prev = position.translation;
            position.translation += (velocity.0 * dt).extend(0.0);
            trace!("position: {:?} -> {:?}", prev, position.translation);
        });

    // Second half-step velocity update
    no_grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &grav_query {
                let force =
                    compute_gravitational_force(&position, mass, other_position, other_mass);
                net_force += force;
            }

            velocity.0 += net_force / mass.0 * half_dt;
        });

    // Handle interactions between bodies with gravity and each other
    // We need to clone the original gravitational body query because we are doing
    // a double iteration over it, which is not allowed by rust
    let original_grav_query = grav_query
        .iter()
        .map(|(mass, velocity, transform)| (*mass, *velocity, *transform))
        .collect::<Vec<_>>();

    debug!("original_grav_query: {:?}", original_grav_query);

    // First half-step velocity update
    grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &original_grav_query {
                if position
                    .translation
                    .distance_squared(other_position.translation)
                    > MIN_DISTANCE_SQUARED
                {
                    let force =
                        compute_gravitational_force(&position, mass, other_position, other_mass);
                    net_force += force;
                }
            }

            let prev = velocity.0;
            velocity.0 += net_force / mass.0 * half_dt;
            trace!("velocity: {:?} -> {:?}", prev, velocity.0);
        });

    // Full-step position update
    grav_query
        .par_iter_mut()
        .for_each(|(_, velocity, mut position)| {
            let prev = position.translation;
            position.translation += (velocity.0 * dt).extend(0.0);
            trace!("position: {:?} -> {:?}", prev, position.translation);
        });

    // Second half-step velocity update
    grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &original_grav_query {
                if position
                    .translation
                    .distance_squared(other_position.translation)
                    > MIN_DISTANCE_SQUARED
                {
                    let force =
                        compute_gravitational_force(&position, mass, other_position, other_mass);
                    net_force += force;
                }
            }

            let prev = velocity.0;
            velocity.0 += net_force / mass.0 * half_dt;
            trace!("velocity: {:?} -> {:?}", prev, velocity.0);
        });
}

fn compute_gravitational_force(
    pos1: &Transform,
    mass1: &Mass,
    pos2: &Transform,
    mass2: &Mass,
) -> Vec2 {
    debug_assert_ne!(mass1.0, 0.0);
    debug_assert_ne!(mass2.0, 0.0);
    const G: f32 = 100000.0;
    let r = pos2.translation.truncate() - pos1.translation.truncate();
    let mut distance_squared = r.length_squared();
    distance_squared = distance_squared.max(MIN_DISTANCE_SQUARED);

    // Assume a simplified gravitational constant and avoid sqrt for performance
    let force_magnitude = G * mass1.0 * mass2.0 / distance_squared;
    let force_direction = r / distance_squared; // r is already a vector
    let out = force_direction * force_magnitude;
    debug_assert!(out.x.is_finite());
    debug_assert!(out.y.is_finite());
    out
}
