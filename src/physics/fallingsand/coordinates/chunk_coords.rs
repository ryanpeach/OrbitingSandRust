use ggez::glam::Vec2;
use ggez::graphics::Color;
use ggez::graphics::MeshBuilder;
use ggez::graphics::Rect;
use ggez::graphics::Vertex;

use crate::physics::fallingsand::util::OwnedMeshData;

/// A chunk that can be rendered and simulated
pub trait ChunkCoords: Send + Sync {
    /* Drawing */
    fn get_outline(&self) -> Vec<Vec2>;
    fn get_positions(&self, res: u16) -> Vec<Vec2>;
    fn get_indices(&self, res: u16) -> Vec<u32>;
    fn get_uvs(&self, res: u16) -> Vec<Vec2>;
    fn get_vertices(&self, res: u16) -> Vec<Vertex> {
        let positions = self.get_positions(res);
        let uvs = self.get_uvs(res);
        let vertexes: Vec<Vertex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(p, uv)| Vertex {
                position: [p.x, p.y],
                uv: [uv.x, uv.y],
                color: [1.0, 1.0, 1.0, 1.0],
            })
            .collect();
        vertexes
    }

    /* Shape Parameter Getters */
    fn get_cell_radius(&self) -> f32;
    fn get_start_radius(&self) -> f32;
    fn get_end_radius(&self) -> f32;
    fn get_start_radial_theta(&self) -> f32;
    fn get_end_radial_theta(&self) -> f32;
    fn get_num_radial_lines(&self) -> usize;
    fn get_num_concentric_circles(&self) -> usize;
    fn get_bounding_box(&self) -> Rect;
    fn total_size(&self) -> usize {
        self.get_num_radial_lines() * self.get_num_concentric_circles()
    }

    /* Identity */
    fn get_start_concentric_circle_layer_relative(&self) -> usize;
    fn get_start_concentric_circle_absolute(&self) -> usize;
    fn get_end_concentric_circle_absolute(&self) -> usize;
    fn get_end_concentric_circle_relative(&self) -> usize;
    fn get_end_radial_line(&self) -> usize;
    fn get_start_radial_line(&self) -> usize;
    fn get_layer_num(&self) -> usize;

    /* Mesh */
    fn calc_chunk_outline(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let outline = self.get_outline();
        let _ = mb.line(&outline, 0.1, Color::WHITE);
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }

    fn calc_chunk_meshdata(&self, res: u16) -> OwnedMeshData {
        let indices = self.get_indices(res);
        let vertices: Vec<Vertex> = self.get_vertices(res);
        OwnedMeshData {
            vertices,
            indices,
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }

    fn calc_chunk_triangle_wireframe(&self, res: u16) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let indices = self.get_indices(res);
        let vertices: Vec<Vertex> = self.get_vertices(res);
        for i in (0..indices.len()).step_by(3) {
            let i1: usize = indices[i] as usize;
            let i2 = indices[i + 1] as usize;
            let i3 = indices[i + 2] as usize;

            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;
            let p3 = vertices[i3].position;

            let _ = mb.line(&[p1, p2, p3, p1], 0.1, Color::WHITE).unwrap();
        }
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }

    fn calc_chunk_uv_wireframe(&self, res: u16) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let indices = self.get_indices(res);
        let vertices: Vec<Vertex> = self.get_vertices(res);
        for i in (0..indices.len()).step_by(3) {
            let i1 = indices[i] as usize;
            let i2 = indices[i + 1] as usize;
            let i3 = indices[i + 2] as usize;

            let p1 = vertices[i1].uv;
            let p1_multiplied = Vec2::new(p1[0] * 10.0, p1[1] * 10.0);
            let p2 = vertices[i2].uv;
            let p2_multiplied = Vec2::new(p2[0] * 10.0, p2[1] * 10.0);
            let p3 = vertices[i3].uv;
            let p3_multiplied = Vec2::new(p3[0] * 10.0, p3[1] * 10.0);

            let _ = mb
                .line(
                    &[p1_multiplied, p2_multiplied, p3_multiplied, p1_multiplied],
                    0.1,
                    Color::WHITE,
                )
                .unwrap();
        }
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }
}
