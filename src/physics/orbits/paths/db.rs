//! A database of positions
//! Used for n-body simulation and collision detection

use std::time::Duration;

use bevy::ecs::{component::Component, entity::Entity, system::Resource};
use knn::PointCloud;
use quadtree_rs::Quadtree;

use crate::physics::util::{clock::Clock, vectors::WorldCoord};

/// A database of positions
/// Imagine that this is a "point in time" for all orbiting bodies
/// If you have the point in time, you can query the database for:
///
/// * The k nearest neighbors to any point, this is useful for n-body simulation
/// * Whether or not a point is within a certain radius of any other point, this is useful for collision detection
///
/// WARNING: No two Entites can have the same position at the same time.
///
#[derive(Component)]
struct PositionDatabase {
    time: Clock,
    quadtree: Quadtree<u16, Entity>,
    knn: PointCloud<'static, WorldCoord>,
}

/// A server that holds all the position databases
/// Imagine that this is now itself a Path database. Each PositionDatabase is a "pointcloud" in a database of paths that have the same time delta.
#[derive(Resource)]
struct PathServer {
    start_time: Clock,
    time_delta: Duration,
    all_databases: Vec<PositionDatabase>,
}
