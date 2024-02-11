//! Mesh utilities
//! I found it useful to write my own mesh class in ggez and it has been useful in bevy as well
//! keeps us from having to use specific bevy types in the physics engine
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::ecs::component::Component;

use bevy::math::{Rect, Vec2};

use bevy::{
    asset::{Assets, Handle},
    ecs::system::ResMut,
    gizmos::gizmos::Gizmos,
    render::{
        color::Color,
        mesh::{Indices, Mesh, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
    transform::components::Transform,
};

use crate::physics::util::vectors::Vertex;

/// Useful for frustum culling
/// The bounding box of the mesh to determine if it is visible on the screen
#[derive(Component)]
pub struct MeshBoundingBox(pub Rect);

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
    /// Create a new GizmoDrawableLoop
    pub fn new(mesh: OwnedMeshData, color: Color) -> Self {
        Self { mesh, color }
    }

    /// Draws the mesh using bevy's gizmos, which is an immediate mode renderer
    /// This is useful for chunk outlines and for the brush
    /// This draw mode "loops" like you would for an enclosed shape
    pub fn draw_bevy_gizmo_loop(&self, gizmos: &mut Gizmos, transform: &Transform) {
        for idx in 0..(self.mesh.indices.len() - 1) {
            let idx0 = self.mesh.indices[idx] as usize;
            let idx1 = self.mesh.indices[idx + 1] as usize;
            self.mesh
                .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
        }
        // Now the final line to close the loop
        let idx0 = self.mesh.indices[self.mesh.indices.len() - 1] as usize;
        let idx1 = self.mesh.indices[0] as usize;
        self.mesh
            .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
    }
}

/// A mesh that can be drawn using bevy's gizmos (immediate mode renderer)
/// This version draws the mesh using triangles
/// This is useful for wireframes
#[derive(Component)]
pub struct GizmoDrawableTriangles {
    /// The mesh to draw
    pub mesh: OwnedMeshData,
    /// The color to draw the mesh
    pub color: Color,
}

impl GizmoDrawableTriangles {
    /// Create a new GizmoDrawableTriangles
    pub fn new(mesh: OwnedMeshData, color: Color) -> Self {
        Self { mesh, color }
    }

    /// Draws the mesh using bevy's gizmos, which is an immediate mode renderer
    /// This is useful for wireframes
    /// This draw mode draws each triangle (triple) individually
    pub fn draw_bevy_gizmo_triangles(&self, gizmos: &mut Gizmos, transform: &Transform) {
        for idx in (0..self.mesh.indices.len()).step_by(3) {
            let idx0 = self.mesh.indices[idx] as usize;
            let idx1 = self.mesh.indices[idx + 1] as usize;
            let idx2 = self.mesh.indices[idx + 2] as usize;
            self.mesh
                .draw_bevy_gizmo_line(idx0, idx1, transform, gizmos, self.color);
            self.mesh
                .draw_bevy_gizmo_line(idx1, idx2, transform, gizmos, self.color);
            self.mesh
                .draw_bevy_gizmo_line(idx2, idx0, transform, gizmos, self.color);
        }
    }
}

/// Represents a mesh that is owned by this object
/// For some reason a MeshData in ggez object has a lifetime and is a set of borrows.
/// This is a workaround for that.
#[derive(Clone)]
pub struct OwnedMeshData {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex>,
    /// The indices of the mesh, relating to the vertices
    pub indices: Vec<u32>,
}

/// Create an empty OwnedMeshData
impl Default for OwnedMeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl OwnedMeshData {
    /// Create a new OwnedMeshData object
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Get the uv bounds of a list of vertices
    pub fn calc_bounds(&self) -> MeshBoundingBox {
        let width: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[0])
            .fold(0.0, |a, b| a.max(b));
        let height: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[1])
            .fold(0.0, |a, b| a.max(b));
        let min_x: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[0])
            .fold(f32::INFINITY, |a, b| a.min(b));
        let min_y: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[1])
            .fold(f32::INFINITY, |a, b| a.min(b));
        MeshBoundingBox(Rect::new(min_x, min_y, width, height))
    }

    /// Loads the mesh into bevy's asset system and returns a handle to it
    pub fn load_bevy_mesh(&self, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // Assuming that Vertex struct has position, uv, and color fields
        let positions: Vec<[f32; 3]> = self
            .vertices
            .iter()
            .map(|v| {
                [v.position.x, v.position.y, 0.0] // Bevy's mesh uses Vec3 for position
            })
            .collect();

        let uvs: Vec<[f32; 2]> = self.vertices.iter().map(|v| [v.uv.x, v.uv.y]).collect();

        let colors: Vec<[f32; 4]> = self
            .vertices
            .iter()
            .map(|v| [v.color.r(), v.color.g(), v.color.b(), v.color.a()])
            .collect();

        // Set vertex positions, UVs, and colors
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4(colors),
        );

        // Set indices
        mesh.set_indices(Some(Indices::U32(self.indices.clone())));

        meshes.add(mesh)
    }

    /// Simply draws a line from an index to another but applies the transform first
    fn draw_bevy_gizmo_line(
        &self,
        idx0: usize,
        idx1: usize,
        transform: &Transform,
        gizmos: &mut Gizmos,
        color: Color,
    ) {
        let mut pos0 = self.vertices[idx0].position;
        let mut pos1 = self.vertices[idx1].position;
        pos0.x += transform.translation.x;
        pos0.y += transform.translation.y;
        pos1.x += transform.translation.x;
        pos1.y += transform.translation.y;
        pos0.x *= transform.scale.x;
        pos0.y *= transform.scale.y;
        gizmos.line_2d(Vec2::new(pos0.x, pos0.y), Vec2::new(pos1.x, pos1.y), color);
    }
}
