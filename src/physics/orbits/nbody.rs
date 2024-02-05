use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    math::{Vec2, Vec3Swizzles},
    time::Time,
    transform::components::Transform,
};

use super::components::{GravitationalField, Mass, Velocity};

/// It's important that we don't compute the gravitational force between two bodies that are too
/// close together, because the force will be very large and the simulation will be unstable.
pub const MIN_DISTANCE_SQUARED: f32 = 100.0;
/// The gravitational constant
/// N⋅m^2/kg^2
pub const G: f32 = 1.0e3;

/// The force applied to an entity with its direction
#[derive(Component, Debug, Clone, Copy)]
pub struct ForceVec(pub Vec2);

/// The force applied to an entity
#[derive(Component, Debug, Clone, Copy)]
pub struct Force(pub f32);

/// The force applied to an entity
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

/// The acceleration due to gravity
#[derive(Component, Debug, Clone, Copy)]
pub struct GravitationalAcceleration(pub f32);

impl GravitationalAcceleration {
    /// Returns the acceleration due to gravity towards a mass
    pub fn from_total_mass(total_mass: Mass) -> Self {
        GravitationalAcceleration(G * total_mass.0)
    }
}

/// Just a namespace for the fundamental gravity functions
struct GravityCalculations;

impl GravityCalculations {
    pub fn compute_gravitational_force(
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
        ForceVec((force_direction * force_magnitude).xy())
    }

    /// Updates the velocity of the entity one half step
    pub fn half_step_velocity_update(
        this_body: (Entity, &Transform, &mut Velocity, &Mass),
        other_bodies: &[(Entity, Transform, Velocity, Mass)],
        dt: f32,
    ) {
        let mut net_force = Vec2::ZERO;
        for other_body in other_bodies {
            if this_body.0 == other_body.0 {
                continue;
            }
            let force = Self::compute_gravitational_force(
                this_body.1,
                this_body.3,
                &other_body.1,
                &other_body.3,
            );
            net_force += force.0;
        }
        let vdiff = net_force / this_body.3 .0 * (dt / 2.0);
        this_body.2 .0 += vdiff;
    }

    /// Updates the position of the entity one full step
    pub fn full_position_update(this_body: (Entity, &mut Transform, &Velocity, &Mass), dt: f32) {
        let pdiff = (this_body.2 .0 * dt).extend(0.0);
        this_body.1.translation += pdiff;
    }
}

/// Plugin to set up nbody physics
pub struct NBodyPlugin;

impl Plugin for NBodyPlugin {
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
                GravityCalculations::half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
                GravityCalculations::full_position_update(
                    (entity, &mut transform, &velocity, mass),
                    dt,
                );
                GravityCalculations::half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
            });
    }

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
                GravityCalculations::half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
                GravityCalculations::full_position_update(
                    (entity, &mut transform, &velocity, mass),
                    dt,
                );
                GravityCalculations::half_step_velocity_update(
                    (entity, &transform, &mut velocity, mass),
                    &grav_bodies_copy,
                    dt,
                );
            });
    }
}
