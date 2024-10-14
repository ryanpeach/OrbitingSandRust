use super::vectors::WorldCoord;
use bevy::log::info;
use bevy::{
    math::Vec2,
    prelude::{Camera2d, Query},
    render::camera::{Camera, OrthographicProjection},
    transform::components::GlobalTransform,
    window::Window,
};

/// Take a mouse coordinate and translate it into a [`WorldCoord`] based on the [`Camera2d`]'s
/// [`GlobalTransform`]
#[must_use]
pub fn get_mouse_world_position(
    window: &Window,
    camera: (&Camera, &GlobalTransform),
) -> Option<WorldCoord> {
    if let Some(cursor_pos) = window.cursor_position() {
        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.0.viewport_to_world(camera.1, cursor))
            .map(|ray| ray.origin.truncate())
        {
            return Some(world_position.into());
        }
    }
    None
}
