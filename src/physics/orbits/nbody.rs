use bevy::prelude::*;

use super::components::{GravitationalField, Mass, Velocity};

pub fn leapfrog_integration_system(
    mut no_grav_query: Query<(&Mass, &mut Velocity, &mut Transform), Without<GravitationalField>>,
    mut grav_query: Query<(&Mass, &mut Velocity, &mut Transform), With<GravitationalField>>,
    time: Res<Time>,
) {
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
            position.translation += (velocity.0 * dt).extend(0.0);
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

    // First half-step velocity update
    grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &original_grav_query {
                if position.translation != other_position.translation {
                    let force =
                        compute_gravitational_force(&position, mass, other_position, other_mass);
                    net_force += force;
                }
            }

            velocity.0 += net_force / mass.0 * half_dt;
        });

    // Full-step position update
    grav_query
        .par_iter_mut()
        .for_each(|(_, velocity, mut position)| {
            position.translation += (velocity.0 * dt).extend(0.0);
        });

    // Second half-step velocity update
    grav_query
        .par_iter_mut()
        .for_each(|(mass, mut velocity, position)| {
            let mut net_force = Vec2::ZERO;

            for (other_mass, _, other_position) in &original_grav_query {
                if position.translation != other_position.translation {
                    let force =
                        compute_gravitational_force(&position, mass, other_position, other_mass);
                    net_force += force;
                }
            }

            velocity.0 += net_force / mass.0 * half_dt;
        });
}

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
const MIN_DISTANCE_SQUARED: f32 = 1.0;

fn compute_gravitational_force(
    pos1: &Transform,
    mass1: &Mass,
    pos2: &Transform,
    mass2: &Mass,
) -> Vec2 {
    let r = pos2.translation.truncate() - pos1.translation.truncate();
    let mut distance_squared = r.length_squared();
    distance_squared = distance_squared.max(MIN_DISTANCE_SQUARED);

    // Assume a simplified gravitational constant and avoid sqrt for performance
    let force_magnitude = mass1.0 * mass2.0 / distance_squared;
    let force_direction = r / distance_squared; // r is already a vector
    force_direction * force_magnitude
}
