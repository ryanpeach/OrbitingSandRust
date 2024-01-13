use bevy::ecs::{bundle::Bundle, component::Component, entity::Entity, system::Resource};
use quadtree_rs::Quadtree;

use crate::physics::util::clock::InGameTime;

#[derive(Component)]
struct EntityQuadtree {
    quadtree: Quadtree<u16, Entity>,
}

#[derive(Bundle)]
struct PositionDatabase {
    start_time: InGameTime,
    quadtree: EntityQuadtree,
}

#[derive(Resource)]
struct PathServer {
    all_quadtrees: Vec<PositionDatabase>,
}
