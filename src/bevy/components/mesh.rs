//! Mesh utilities
//! I found it useful to write my own mesh class in ggez and it has been useful in bevy as well
//! keeps us from having to use specific bevy types in the physics engine
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::color::Color;
use bevy::ecs::component::Component;

use bevy::math::Rect;

use bevy::{gizmos::gizmos::Gizmos, transform::components::Transform};

use crate::common::util::mesh::OwnedMeshData;

/// Useful for frustum culling
/// The bounding box of the mesh to determine if it is visible on the screen
#[derive(Component)]
pub struct MeshBoundingBox(pub Rect);
