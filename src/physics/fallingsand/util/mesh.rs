use ggez::{
    glam::Vec2,
    graphics::{MeshData, Rect, Vertex},
};

/// Represents a square in 2D space
#[derive(Clone)]
pub struct Square {
    tl: Vec2,
    tr: Vec2,
    bl: Vec2,
    br: Vec2,
}

impl Square {
    /// Create a new square from the four corners
    pub fn new(tl: Vec2, tr: Vec2, bl: Vec2, br: Vec2) -> Self {
        Self { tl, tr, bl, br }
    }
    /// Create a new square from the top left corner and the width and height (hw stands for height width)
    pub fn new_hw(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            tl: Vec2::new(x, y),
            tr: Vec2::new(x + w, y),
            bl: Vec2::new(x, y + h),
            br: Vec2::new(x + w, y + h),
        }
    }
}

/// Represents a mesh that is owned by this object
/// For some reason a MeshData object has a lifetime and is a set of borrows.
/// This is a workaround for that.
#[derive(Clone)]
pub struct OwnedMeshData {
    vertexes: Vec<Vertex>,
    indices: Vec<u32>,
}

/// Create an empty OwnedMeshData
impl Default for OwnedMeshData {
    fn default() -> Self {
        Self {
            vertexes: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl OwnedMeshData {
    pub fn new(positions: Vec<Square>, uvs: Vec<Square>) -> Self {
        let vertexes = OwnedMeshData::calc_vertexes(positions, uvs);
        let indices = OwnedMeshData::calc_indices(positions.len());
        Self { vertexes, indices }
    }

    pub fn calc_vertexes(positions: Vec<Square>, uvs: Vec<Square>) -> Vec<Vertex> {
        debug_assert_eq!(positions.len(), uvs.len());
        let mut result = Vec::with_capacity(positions.len() * 4);
        for i in 0..positions.len() {
            result.push(Vertex {
                position: positions[i].tl.to_array(),
                uv: uvs[i].tl.to_array(),
                color: [1.0, 1.0, 1.0, 1.0],
            });
            result.push(Vertex {
                position: positions[i].tr.to_array(),
                uv: uvs[i].tr.to_array(),
                color: [1.0, 1.0, 1.0, 1.0],
            });
            result.push(Vertex {
                position: positions[i].bl.to_array(),
                uv: uvs[i].bl.to_array(),
                color: [1.0, 1.0, 1.0, 1.0],
            });
            result.push(Vertex {
                position: positions[i].br.to_array(),
                uv: uvs[i].br.to_array(),
                color: [1.0, 1.0, 1.0, 1.0],
            });
        }
        result
    }

    pub fn calc_indices(num_squares: usize) -> Vec<u32> {
        let mut result = Vec::with_capacity(num_squares * 6);
        for i in 0..num_squares {
            let offset = i as u32 * 4;
            result.push(offset);
            result.push(offset + 1);
            result.push(offset + 2);
            result.push(offset + 1);
            result.push(offset + 2);
            result.push(offset + 3);
        }
        result
    }
}

impl OwnedMeshData {
    /// Convert to a ggez MeshData object
    /// which takes references and has a lifetime
    pub fn to_mesh_data(&self) -> MeshData {
        MeshData {
            vertices: &self.vertexes,
            indices: &self.indices,
        }
    }
}
