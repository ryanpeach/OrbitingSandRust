use bevy::ecs::component::Component;
use glam::Vec2;

use crate::physics::util::clock::InGameTime;

/// A path is a series of points that an object will follow. \
/// The object will move from one point to the next in a straight line,
/// and will move at a constant speed.
#[derive(Component)]
pub struct Path {
    path: Vec<Vec2>,
    start_time: InGameTime,
}
