use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, Mesh, MeshData, Rect, Vertex};
use ggez::{graphics, Context};

use crate::physics::fallingsand::chunks::radial_mesh::RadialMesh;

#[derive(Copy, Clone, PartialEq)]
pub enum DrawMode {
    TexturedMesh,
    TriangleWireframe,
    UVWireframe,
}

pub struct Celestial {
    radial_mesh: RadialMesh,
    all_vertices: Vec<Vec<Vertex>>,
    all_indices: Vec<Vec<u32>>,
    all_outlines: Vec<Vec<Vec2>>,
    bounding_boxes: Vec<Rect>,
    draw_mode: DrawMode,
}

impl Celestial {
    pub fn new(radial_mesh: RadialMesh) -> Self {
        let all_vertices = radial_mesh.get_vertexes();
        let all_indices = radial_mesh.get_indices();
        let all_outlines = radial_mesh.get_outlines();
        let bounding_boxes = radial_mesh.get_chunk_bounding_boxes();
        Self {
            radial_mesh,
            all_vertices,
            all_indices,
            all_outlines,
            bounding_boxes,
            draw_mode: DrawMode::TexturedMesh,
        }
    }
    pub fn get_num_chunks(&self) -> usize {
        self.radial_mesh.get_num_chunks()
    }
    pub fn get_all_vertices(&self) -> &Vec<Vec<Vertex>> {
        &self.all_vertices
    }
    pub fn get_all_indices(&self) -> &Vec<Vec<u32>> {
        &self.all_indices
    }
    pub fn get_all_outlines(&self) -> &Vec<Vec<Vec2>> {
        &self.all_outlines
    }
    pub fn get_all_bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
    }
    pub fn get_res(&self) -> u16 {
        self.radial_mesh.get_res()
    }
    pub fn get_chunk_bounding_box(&self, chunk_idx: usize) -> Rect {
        self.bounding_boxes[chunk_idx]
    }
    pub fn get_draw_mode(&self) -> DrawMode {
        self.draw_mode
    }
    pub fn set_draw_mode(&mut self, draw_mode: DrawMode) {
        self.draw_mode = draw_mode;
    }

    // Changes the resoution of the mesh and recomputes all vertices, indices, outlines, and bounding boxes
    pub fn set_res(&mut self, res: u16) {
        self.radial_mesh.set_res(res);
        self.all_vertices = self.radial_mesh.get_vertexes();
        self.all_indices = self.radial_mesh.get_indices();
        self.all_outlines = self.radial_mesh.get_outlines();
        self.bounding_boxes = self.radial_mesh.get_chunk_bounding_boxes();
    }

    // ============================
    // Drawing
    // ============================
    pub fn draw_chunk(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        chunk_idx: usize,
        draw_params: graphics::DrawParam,
    ) {
        match self.draw_mode {
            DrawMode::TexturedMesh => {
                self.draw_chunk_textured_mesh(ctx, canvas, chunk_idx, draw_params)
            }
            DrawMode::TriangleWireframe => {
                self.draw_chunk_triangle_wireframe(ctx, canvas, chunk_idx, draw_params)
            }
            DrawMode::UVWireframe => {
                self.draw_chunk_uv_wireframe(ctx, canvas, chunk_idx, draw_params)
            }
        }
    }

    pub fn draw_chunk_textured_mesh(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        chunk_idx: usize,
        draw_params: graphics::DrawParam,
    ) {
        let texture = self.radial_mesh.get_texture(ctx, chunk_idx);
        let mesh_data = MeshData {
            vertices: &self.all_vertices[chunk_idx],
            indices: &self.all_indices[chunk_idx][..],
        };
        let mesh = Mesh::from_data(ctx, mesh_data);
        canvas.draw_textured_mesh(mesh, texture, draw_params);
    }

    pub fn draw_chunk_triangle_wireframe(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        chunk_idx: usize,
        draw_params: graphics::DrawParam,
    ) {
        for i in (0..self.all_indices[chunk_idx].len()).step_by(3) {
            let i1: usize = self.all_indices[chunk_idx][i] as usize;
            let i2 = self.all_indices[chunk_idx][i + 1] as usize;
            let i3 = self.all_indices[chunk_idx][i + 2] as usize;

            let p1 = self.all_vertices[chunk_idx][i1].position;
            let p2 = self.all_vertices[chunk_idx][i2].position;
            let p3 = self.all_vertices[chunk_idx][i3].position;

            canvas.draw(
                &Mesh::new_line(ctx, &[p1, p2, p3, p1], 0.1, Color::WHITE).unwrap(),
                draw_params,
            );
        }
    }

    pub fn draw_chunk_uv_wireframe(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        chunk_idx: usize,
        draw_params: graphics::DrawParam,
    ) {
        for i in (0..self.all_indices[chunk_idx].len()).step_by(3) {
            let i1 = self.all_indices[chunk_idx][i] as usize;
            let i2 = self.all_indices[chunk_idx][i + 1] as usize;
            let i3 = self.all_indices[chunk_idx][i + 2] as usize;

            let p1 = self.all_vertices[chunk_idx][i1].uv;
            let p1_multiplied = Vec2::new(p1[0] * 10.0, p1[1] * 10.0);
            let p2 = self.all_vertices[chunk_idx][i2].uv;
            let p2_multiplied = Vec2::new(p2[0] * 10.0, p2[1] * 10.0);
            let p3 = self.all_vertices[chunk_idx][i3].uv;
            let p3_multiplied = Vec2::new(p3[0] * 10.0, p3[1] * 10.0);

            canvas.draw(
                &Mesh::new_line(
                    ctx,
                    &[p1_multiplied, p2_multiplied, p3_multiplied, p1_multiplied],
                    0.1,
                    Color::WHITE,
                )
                .unwrap(),
                draw_params,
            );
        }
    }
}
