use ggez::graphics::{MeshData, Rect, Vertex};

/// Represents a mesh that is owned by this object
/// For some reason a MeshData object has a lifetime and is a set of borrows.
/// This is a workaround for that.
pub struct OwnedMeshData {
    pub uv_bounds: Rect,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Create an empty OwnedMeshData
impl Default for OwnedMeshData {
    fn default() -> Self {
        Self {
            uv_bounds: Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl OwnedMeshData {
    /// Convert to a ggez MeshData object
    /// which takes references and has a lifetime
    pub fn to_mesh_data(&self) -> MeshData {
        MeshData {
            vertices: &self.vertices,
            indices: self.indices.as_slice(),
        }
    }

    /// Combine a list of OwnedMeshData objects into one OwnedMeshData object
    /// This dramatically increases draw performance in testing.
    /// Remarks on Implementation:
    /// * You need to add the previous last_idx to all the elements of the next indices
    /// * You also need to un_normalize the uvs and then re_normalize them at the end
    pub fn combine(lst: &Vec<OwnedMeshData>) -> OwnedMeshData {
        let mut combined_vertices = Vec::new();
        let mut combined_indices = Vec::new();

        // This is to find the max and min bounds for the UVs
        let width: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.w + mesh.uv_bounds.x)
            .fold(0.0, |a, b| a.max(b));
        let height: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.h + mesh.uv_bounds.y)
            .fold(0.0, |a, b| a.max(b));
        let min_x: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.x)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let min_y: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.y)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let max_x: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.x + mesh.uv_bounds.w)
            .fold(0.0, |a, b| a.max(b));
        let max_y: f32 = lst
            .iter()
            .map(|mesh| mesh.uv_bounds.y + mesh.uv_bounds.h)
            .fold(0.0, |a, b| a.max(b));

        let mut last_idx = 0usize;
        for mesh_data in lst {
            let mut new_vertices = Vec::with_capacity(mesh_data.vertices.len());
            for vertex in &mesh_data.vertices {
                let un_normalized_u =
                    (vertex.uv[0] * mesh_data.uv_bounds.w + mesh_data.uv_bounds.x) / max_x;
                let un_normalized_v =
                    (vertex.uv[1] * mesh_data.uv_bounds.h + mesh_data.uv_bounds.y) / max_y;
                new_vertices.push(Vertex {
                    position: vertex.position,
                    uv: [un_normalized_u, un_normalized_v],
                    color: vertex.color,
                })
            }

            let mut new_indices = Vec::with_capacity(mesh_data.indices.len());
            for index in &mesh_data.indices {
                new_indices.push(index + last_idx as u32);
            }

            last_idx += new_vertices.len();
            combined_vertices.extend(new_vertices);
            combined_indices.extend(new_indices);
        }

        OwnedMeshData {
            uv_bounds: Rect::new(min_x, min_y, width, height),
            vertices: combined_vertices,
            indices: combined_indices,
        }
    }
}
