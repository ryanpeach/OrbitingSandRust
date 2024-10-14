use bevy::{
    asset::{Assets, Handle},
    color::{Color, ColorToComponents},
    math::{Rect, Vec2, Vec3},
    prelude::{Component, Gizmos, ResMut},
    render::{
        mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    transform::components::Transform,
};

/// Useful for frustum culling
/// The bounding box of the mesh to determine if it is visible on the screen
#[derive(Component)]
pub struct MeshBoundingBox(pub Rect);

/// A vertex in a mesh
/// Originally from ggez
/// TODO: move to bevy's types
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    /// The position of the vertex
    pub position: Vec2,
    /// The texture coordinates of the vertex
    pub uv: Vec2,
    /// The color of the vertex
    pub color: Color,
}

/// Represents a mesh that is owned by this object
/// For some reason a `MeshData` in ggez object has a lifetime and is a set of borrows.
/// This is a workaround for that.
#[derive(Clone)]
pub struct OwnedMeshData {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex>,
    /// The indices of the mesh, relating to the vertices
    pub indices: Vec<u32>,
}

/// Create an empty `OwnedMeshData`
impl Default for OwnedMeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl OwnedMeshData {
    /// Create a new `OwnedMeshData` object
    #[must_use]
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

impl OwnedMeshData {
    /// Get the uv bounds of a list of vertices
    #[must_use]
    pub fn bounds(&self) -> MeshBoundingBox {
        let width: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[0])
            .fold(0.0, f32::max);
        let height: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[1])
            .fold(0.0, f32::max);
        let min_x: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[0])
            .fold(f32::INFINITY, f32::min);
        let min_y: f32 = self
            .vertices
            .iter()
            .map(|vertex| vertex.uv[1])
            .fold(f32::INFINITY, f32::min);
        MeshBoundingBox(Rect::new(min_x, min_y, width, height))
    }

    /// Loads the mesh into bevy's asset system and returns a handle to it
    pub fn load_bevy_mesh(&self, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

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
            .map(|v| v.color.to_srgba().to_f32_array())
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
        mesh.insert_indices(Indices::U32(self.indices.clone()));

        meshes.add(mesh)
    }
}
