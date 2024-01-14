use bevy::{
    asset::{Assets, Handle},
    ecs::system::ResMut,
    render::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use glam::Vec2;

use crate::physics::util::vectors::{Rect, Vertex};

use super::grid::Grid;

/// Represents a mesh that is owned by this object
/// For some reason a MeshData object has a lifetime and is a set of borrows.
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

    // /// Convert to a ggez MeshData object
    // /// which takes references and has a lifetime
    // pub fn to_mesh_data(&self) -> MeshData {
    //     MeshData {
    //         vertices: &self.vertices,
    //         indices: self.indices.as_slice(),
    //     }
    // }

    /// Combine a list of OwnedMeshData objects into one OwnedMeshData object
    /// This dramatically increases draw performance in testing.
    /// Remarks on Implementation:
    /// * You need to add the previous last_idx to all the elements of the next indices
    /// * You also need to un_normalize the uvs and then re_normalize them at the end
    pub fn combine(vec_grid: &[Grid<OwnedMeshData>]) -> OwnedMeshData {
        let mut combined_vertices = Vec::new();
        let mut combined_indices = Vec::new();
        let lst = vec_grid.iter().flatten().collect::<Vec<&OwnedMeshData>>();

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
                    uv: Vec2::new(un_normalized_u, un_normalized_v),
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
    use super::*;
    use crate::physics::fallingsand::data::element_directory::ElementGridDir;
    use crate::physics::fallingsand::elements::{element::Element, sand::Sand, vacuum::Vacuum};
    use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
    use crate::physics::fallingsand::util::enums::MeshDrawMode;

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

    #[test]
    fn test_combine() {
        let meshes = get_element_grid_dir()
            .get_coordinate_dir()
            .get_mesh_data(MeshDrawMode::TexturedMesh);
        let combined_mesh = OwnedMeshData::combine(&meshes);
        // Test that the combined_mesh uvs are normalized
        for vertex in &combined_mesh.vertices {
            assert!(vertex.uv[0] <= 1.0);
            assert!(vertex.uv[0] >= 0.0);
            assert!(vertex.uv[1] <= 1.0);
            assert!(vertex.uv[1] >= 0.0);
        }
        // Test that the length of the combined_mesh indices is the same as the sum of the lengths of the meshes
        let mut sum_indices = 0;
        let mut sum_vertices = 0;
        for grid in &meshes {
            for mesh in grid {
                sum_indices += mesh.indices.len();
                sum_vertices += mesh.vertices.len();
            }
        }
        assert_eq!(combined_mesh.indices.len(), sum_indices);
        assert_eq!(combined_mesh.vertices.len(), sum_vertices);
        // Test that the indices have been offset correctly
        assert_eq!(*combined_mesh.indices.iter().min().unwrap(), 0u32);
        assert_eq!(
            *combined_mesh.indices.iter().max().unwrap(),
            (combined_mesh.vertices.iter().len() - 1) as u32
        );
    }
}
