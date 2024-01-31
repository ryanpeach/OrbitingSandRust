use bevy::{
    ecs::{component::Component, system::Query},
    math::Vec2,
    render::extract_component::ExtractComponent,
    transform::components::Transform,
};

#[derive(ExtractComponent, Component, Debug, Clone, Copy)]
pub struct GravitationalField;

#[derive(ExtractComponent, Component, Debug, Clone, Copy)]
pub struct Mass(pub f32);

#[derive(ExtractComponent, Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

/// No need to use this everywhere, Transform will do for most things
/// But this is useful for the compute shader because it is extractable
/// A position that is extractable for the compute shader
#[derive(ExtractComponent, Component, Debug, Clone, Copy)]
pub struct OrbitalPosition(pub Vec2);

impl OrbitalPosition {
    pub fn new(vec: Vec2) -> Self {
        Self(vec)
    }

    pub fn zero() -> Self {
        Self(Vec2::ZERO)
    }

    pub fn transform(&self) -> Transform {
        Transform::from_translation(self.0.extend(0.0))
    }

    pub fn sync_with_transform_system(mut query: Query<(&OrbitalPosition, &mut Transform)>) {
        query.par_iter_mut().for_each(|(pos, mut transform)| {
            transform.translation = pos.0.extend(0.0);
        });
    }
}
