//! Mesh utilities
//! I found it useful to write my own mesh class in ggez and it has been useful in bevy as well
//! keeps us from having to use specific bevy types in the physics engine
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::color::Color;
use bevy::ecs::component::Component;

use bevy::{gizmos::gizmos::Gizmos, transform::components::Transform};

use crate::common::util::mesh::OwnedMeshData;

/// A mesh that can be drawn using bevy's gizmos (immediate mode renderer)
/// This version draws the mesh using lines and assumes that the mesh is a loop
/// This is useful for chunk outlines and for the brush
#[derive(Component)]
pub struct GizmoDrawableLoop {
    /// The mesh to draw
    pub mesh: OwnedMeshData,
    /// The color to draw the mesh
    pub color: Color,
}

impl GizmoDrawableLoop {
    /// Create a new `GizmoDrawableLoop`
    #[must_use]
    pub fn new(mesh: OwnedMeshData, color: Color) -> Self {
        Self { mesh, color }
    }

    /// Draws the mesh using bevy's gizmos, which is an immediate mode renderer
    /// This is useful for chunk outlines and for the brush
    /// This draw mode "loops" like you would for an enclosed shape
    pub fn draw_bevy_gizmo_loop(&self, gizmos: &mut Gizmos, transform: &Transform) {
        for idx in 0..(self.mesh.indices.len() - 1) {
            let idx0 = self.mesh.indices[idx];
            let idx1 = self.mesh.indices[idx + 1];
            self.mesh
                .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
        }
        // Now the final line to close the loop
        let idx0 = self.mesh.indices[self.mesh.indices.len() - 1];
        let idx1 = self.mesh.indices[0];
        self.mesh
            .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
    }
}

/// A mesh that can be drawn using bevy's gizmos (immediate mode renderer)
/// This version draws the mesh using triangles
/// This is useful for wireframes
#[derive(Component)]
pub struct GizmoDrawableGrid {
    /// The mesh to draw
    pub mesh: OwnedMeshData,
    /// The color to draw the mesh
    pub color: Color,
}

impl GizmoDrawableGrid {
    /// Create a new `GizmoDrawableTriangles`
    #[must_use]
    pub fn new(mesh: OwnedMeshData, color: Color) -> Self {
        Self { mesh, color }
    }

    /// Draws the mesh using bevy's gizmos, which is an immediate mode renderer
    /// This is useful for wireframes
    /// This draw mode draws each triangle (triple) individually
    pub fn draw_bevy_gizmo_grid(&self, gizmos: &mut Gizmos, transform: &Transform) {
        for idx in (0..self.mesh.indices.len()).step_by(3) {
            let idx0 = self.mesh.indices[idx];
            let idx1 = self.mesh.indices[idx + 1];
            let idx2 = self.mesh.indices[idx + 2];
            if idx % 2 == 0 {
                self.mesh
                    .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
                self.mesh
                    .draw_bevy_gizmo_line(idx2, idx0, transform, gizmos, self.color);
            } else {
                self.mesh
                    .draw_bevy_gizmo_line(idx1, idx2, transform, gizmos, self.color);
            }
        }
    }
}
