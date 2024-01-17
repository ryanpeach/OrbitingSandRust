//! The bevy camera for the game

use bevy::{
    core_pipeline::core_2d::Camera2d,
    ecs::{
        event::EventReader,
        system::{Query, Res},
    },
    input::{
        keyboard::KeyCode,
        mouse::{MouseScrollUnit, MouseWheel},
        Input,
    },
    math::Vec3,
    time::Time,
    transform::components::Transform,
};

/// Bevy Systems
pub fn zoom_camera_system(
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

pub fn move_camera_system(
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
