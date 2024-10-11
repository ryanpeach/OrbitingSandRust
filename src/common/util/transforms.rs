use bevy::{math::Vec2, prelude::Query, window::Window};

use super::vectors::{ModelCoord, ViewCoord};

/// Take a mouse coordinate and translate it into a position relative to the center of the window
/// It's a [`ModelCoord`] because it's relative to the center of the window, not relative to the
/// camera. You would then add this to a cameras translation to get [`ViewCoord`]
#[must_use]
pub fn window_to_model_centered(
    windows: &Query<'_, '_, &mut Window>,
    position: Vec2,
) -> ModelCoord {
    // Translate cursor position to coordinate system with origin at the center of the screen
    let window = windows.single();
    let window_size = Vec2::new(window.width(), window.height());
    let centered_x = position.x - window_size.x / 2.0;
    let centered_y = -(position.y - window_size.y / 2.0);
    ModelCoord(Vec2::new(centered_x, centered_y))
}
