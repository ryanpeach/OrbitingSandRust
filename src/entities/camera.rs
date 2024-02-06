//! The bevy camera for the game

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        system::{Commands, Query, Res},
    },
    input::{
        keyboard::KeyCode,
        mouse::{MouseScrollUnit, MouseWheel},
        Input,
    },
    math::{Rect, Vec2, Vec3},
    render::view::Visibility,
    time::Time,
    transform::components::{GlobalTransform, Transform},
    window::Window,
};

use crate::physics::fallingsand::util::mesh::MeshBoundingBox;

/// A layer in front of the game. Z-index = 1
#[derive(Component, Debug, Default)]
pub struct OverlayLayer1;

/// The plugin for the camera system
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    /// Build the camera plugin
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::zoom_camera_system);
        app.add_systems(Update, Self::move_camera_system);
        app.add_systems(Update, Self::frustum_culling_2d);
    }
}

impl CameraPlugin {
    /// Zoom the camera based on mouse wheel scroll
    fn zoom_camera_system(
        time: Res<Time>,
        mut scroll_evr: EventReader<MouseWheel>,
        mut query: Query<(&mut Transform, &mut Camera2d)>,
    ) {
        let mut delta = 0.;
        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    delta += ev.y;
                }
                MouseScrollUnit::Pixel => {
                    delta += ev.y;
                }
            }
        }
        if delta != 0. {
            for (mut transform, _) in query.iter_mut() {
                transform.scale *= 1. + delta * time.delta_seconds() * 0.5;
            }
        }
    }

    /// Move the camera based on keyboard input
    fn move_camera_system(
        time: Res<Time>,
        keyboard_input: Res<Input<KeyCode>>,
        mut query: Query<(&mut Transform, &mut Camera2d)>,
    ) {
        let mut delta = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::A) {
            delta.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::D) {
            delta.x += 1.;
        }
        if keyboard_input.pressed(KeyCode::W) {
            delta.y += 1.;
        }
        if keyboard_input.pressed(KeyCode::S) {
            delta.y -= 1.;
        }
        if delta != Vec3::ZERO {
            for (mut transform, _) in query.iter_mut() {
                let scale = transform.scale;
                transform.translation += delta * time.delta_seconds() * scale * 100.;
            }
        }
    }

    /// Don't render entities that are not in the camera's frustum
    /// Uses the Visibility component to hide and show entities
    fn frustum_culling_2d(
        mut commands: Commands,
        camera: Query<(&Camera2d, &GlobalTransform)>,
        mut mesh_entities: Query<(Entity, &MeshBoundingBox, &Visibility, &Transform)>,
        windows: Query<&Window>,
    ) {
        let (_, camera_transform) = camera.single();
        let camera_transform = camera_transform.compute_transform();
        let window = windows.single();

        let width = window.resolution.width();
        let height = window.resolution.height();

        // Get the camera rect in world coordinates using the translation and scale
        let camera_rect = Rect::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
            width * camera_transform.scale.x,
            height * camera_transform.scale.y,
        );

        for (entity, mesh_bb, visible, transform) in mesh_entities.iter_mut() {
            let overlaps = rect_overlaps(
                &camera_rect,
                &rect_add(&mesh_bb.0, &transform.translation.truncate()),
            );
            if overlaps && *visible == Visibility::Hidden {
                commands.entity(entity).insert(Visibility::Visible);
            } else if !overlaps && *visible == Visibility::Visible {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
    }
}

/// Check if two rectangles overlap
fn rect_overlaps(this: &Rect, other: &Rect) -> bool {
    this.min.x < other.max.x
        && this.max.x > other.min.x
        && this.min.y < other.max.y
        && this.max.y > other.min.y
}

/// Add a vector to every corner of a rectangle
fn rect_add(this: &Rect, other: &Vec2) -> Rect {
    Rect::new(
        this.min.x + other.x,
        this.min.y + other.y,
        this.max.x + other.x,
        this.max.y + other.y,
    )
}
