use bevy::ecs::{component::Component, entity::Entity, system::Resource};
use quadtree_rs::Quadtree;

use crate::physics::util::clock::Clock;

#[derive(Component)]
struct PositionDatabase {
    start_time: Clock,
    quadtree: Quadtree<u16, Entity>,
}

#[derive(Resource)]
struct PathServer {
    all_quadtrees: Vec<PositionDatabase>,
}
