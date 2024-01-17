//! Mesh utilities
//! I found it useful to write my own mesh class in ggez and it has been useful in bevy as well
//! keeps us from having to use specific bevy types in the physics engine

use bevy::{
    asset::{Assets, Handle},
    ecs::system::ResMut,
    render::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};

use crate::physics::util::vectors::{Rect, Vertex};

/// Represents a mesh that is owned by this object
/// For some reason a MeshData in ggez object has a lifetime and is a set of borrows.
/// This is a workaround for that.
#[derive(Clone)]
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

/// Get the uv bounds of a list of vertices
fn calc_uv_bounds(vertices: &[Vertex]) -> Rect {
    let width: f32 = vertices
        .iter()
        .map(|vertex| vertex.uv[0])
        .fold(0.0, |a, b| a.max(b));
    let height: f32 = vertices
        .iter()
        .map(|vertex| vertex.uv[1])
        .fold(0.0, |a, b| a.max(b));
    let min_x: f32 = vertices
        .iter()
        .map(|vertex| vertex.uv[0])
        .fold(f32::INFINITY, |a, b| a.min(b));
    let min_y: f32 = vertices
        .iter()
        .map(|vertex| vertex.uv[1])
        .fold(f32::INFINITY, |a, b| a.min(b));
    Rect::new(min_x, min_y, width, height)
}

impl OwnedMeshData {
    /// Create a new OwnedMeshData object
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let uv_bounds = calc_uv_bounds(&vertices);
        Self {
            uv_bounds,
            vertices,
            indices,
        }
    }

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
}

#[cfg(test)]
mod tests {

    use crate::physics::fallingsand::data::element_directory::ElementGridDir;
    use crate::physics::fallingsand::elements::{element::Element, sand::Sand, vacuum::Vacuum};
    use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;

    /// The default element grid directory for testing
    fn get_element_grid_dir() -> ElementGridDir {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();
        let fill0: &dyn Element = &Vacuum::default();
        let fill1: &dyn Element = &Sand::default();
        ElementGridDir::new_checkerboard(coordinate_dir, fill0, fill1)
    }

    // #[test]
    // fn test_combine() {
    //     let meshes = get_element_grid_dir()
    //         .get_coordinate_dir()
    //         .get_mesh_data(MeshDrawMode::TexturedMesh);
    //     let combined_mesh = OwnedMeshData::combine(&meshes);
    //     // Test that the combined_mesh uvs are normalized
    //     for vertex in &combined_mesh.vertices {
    //         assert!(vertex.uv[0] <= 1.0);
    //         assert!(vertex.uv[0] >= 0.0);
    //         assert!(vertex.uv[1] <= 1.0);
    //         assert!(vertex.uv[1] >= 0.0);
    //     }
    //     // Test that the length of the combined_mesh indices is the same as the sum of the lengths of the meshes
    //     let mut sum_indices = 0;
    //     let mut sum_vertices = 0;
    //     for grid in &meshes {
    //         for mesh in grid {
    //             sum_indices += mesh.indices.len();
    //             sum_vertices += mesh.vertices.len();
    //         }
    //     }
    //     assert_eq!(combined_mesh.indices.len(), sum_indices);
    //     assert_eq!(combined_mesh.vertices.len(), sum_vertices);
    //     // Test that the indices have been offset correctly
    //     assert_eq!(*combined_mesh.indices.iter().min().unwrap(), 0u32);
    //     assert_eq!(
    //         *combined_mesh.indices.iter().max().unwrap(),
    //         (combined_mesh.vertices.iter().len() - 1) as u32
    //     );
    // }
}
