use crate::physics::fallingsand::chunks::chunk::Chunk;
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::{vec2, vec3, BLUE, RED, WHITE};
use macroquad::texture::{FilterMode, Image, Texture2D};
use std::f32::consts::PI;

pub struct CoreChunk {
    radius: f32,
    num_radial_lines: u32,
}

impl Default for CoreChunk {
    fn default() -> Self {
        Self {
            radius: 1.0,
            num_radial_lines: 6,
        }
    }
}

impl CoreChunk {
    fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        // Outer vertices
        for i in 0..self.num_radial_lines {
            let angle1 = i as f32 * 2.0 * PI / self.num_radial_lines as f32;
            let angle2 = (i + 1) as f32 * 2.0 * PI / self.num_radial_lines as f32;
            let pos0 = vec3(0.0, 0.0, 0.0);
            let pos1 = vec3(self.radius * angle1.cos(), self.radius * angle1.sin(), 0.0);
            let pos2 = vec3(self.radius * angle2.cos(), self.radius * angle2.sin(), 0.0);
            let uv0 = vec2(i as f32 / self.num_radial_lines as f32, 0.0);
            let uv1 = vec2((i + 1) as f32 / self.num_radial_lines as f32, 1.0);
            let uv2 = vec2((i + 1) as f32 / self.num_radial_lines as f32, 0.0);
            vertices.push(Vertex {
                position: pos0,
                uv: uv0,
                color: WHITE,
            });
            vertices.push(Vertex {
                position: pos1,
                uv: uv1,
                color: WHITE,
            });
            vertices.push(Vertex {
                position: pos2,
                uv: uv2,
                color: WHITE,
            });
        }

        vertices
    }

    fn get_indices(&self) -> Vec<u16> {
        let mut indices = Vec::new();

        // Triangles
        for i in (0..(self.num_radial_lines * 3)).step_by(3) {
            indices.push(i as u16);
            indices.push((i + 1) as u16);
            indices.push((i + 2) as u16);
        }

        indices
    }

    /// Right now we are just going to return a checkerboard texture
    fn get_texture(&self) -> Texture2D {
        let mut image = Image::gen_image_color(self.num_radial_lines.try_into().unwrap(), 1, WHITE);
        for i in 0..self.num_radial_lines {
            image.set_pixel(i, 0, if i % 2 == 0 { RED } else { BLUE });
        }
        let tex = Texture2D::from_image(&image);
        tex.set_filter(FilterMode::Nearest);
        tex
    }
}

impl Chunk for CoreChunk {
    fn get_mesh(&self) -> Mesh {
        Mesh {
            vertices: self.get_vertices(),
            indices: self.get_indices(),
            texture: Some(self.get_texture()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vertices() {
        let chunk = CoreChunk::default();
        let vertices = chunk.get_vertices();
        let indices = chunk.get_indices();
        // Triangle 1
        assert_eq!(vertices[0].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[0].uv, vec2(0.0, 0.0));
        assert_eq!(indices[0], 0);
        assert_eq!(vertices[1].position, vec3(1.0, 0.0, 0.0));
        assert_eq!(vertices[1].uv, vec2(1.0 / 6.0, 1.0));
        assert_eq!(indices[1], 1);
        assert_eq!(
            vertices[2].position,
            vec3(
                (1.0 * 2.0 * PI / 6.0).cos(),
                (1.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[2].uv, vec2(1.0 / 6.0, 0.0));
        assert_eq!(indices[2], 2);

        // Triangle 2
        assert_eq!(vertices[3].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[3].uv, vec2(1.0 / 6.0, 0.0));
        assert_eq!(indices[3], 3);
        assert_eq!(
            vertices[4].position,
            vec3(
                (1.0 * 2.0 * PI / 6.0).cos(),
                (1.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[4].uv, vec2(2.0 / 6.0, 1.0));
        assert_eq!(indices[4], 4);
        assert_eq!(
            vertices[5].position,
            vec3(
                (2.0 * 2.0 * PI / 6.0).cos(),
                (2.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[5].uv, vec2(2.0 / 6.0, 0.0));
        assert_eq!(indices[5], 5);

        // Triangle 3
        assert_eq!(vertices[6].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[6].uv, vec2(2.0 / 6.0, 0.0));
        assert_eq!(indices[6], 6);
        assert_eq!(
            vertices[7].position,
            vec3(
                (2.0 * 2.0 * PI / 6.0).cos(),
                (2.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[7].uv, vec2(3.0 / 6.0, 1.0));
        assert_eq!(indices[7], 7);
        assert_eq!(
            vertices[8].position,
            vec3(
                (3.0 * 2.0 * PI / 6.0).cos(),
                (3.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[8].uv, vec2(3.0 / 6.0, 0.0));
        assert_eq!(indices[8], 8);

        // Triangle 4
        assert_eq!(vertices[9].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[9].uv, vec2(3.0 / 6.0, 0.0));
        assert_eq!(indices[9], 9);
        assert_eq!(
            vertices[10].position,
            vec3(
                (3.0 * 2.0 * PI / 6.0).cos(),
                (3.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[10].uv, vec2(4.0 / 6.0, 1.0));
        assert_eq!(indices[10], 10);
        assert_eq!(
            vertices[11].position,
            vec3(
                (4.0 * 2.0 * PI / 6.0).cos(),
                (4.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[11].uv, vec2(4.0 / 6.0, 0.0));
        assert_eq!(indices[11], 11);

        // Triangle 5
        assert_eq!(vertices[12].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[12].uv, vec2(4.0 / 6.0, 0.0));
        assert_eq!(indices[12], 12);
        assert_eq!(
            vertices[13].position,
            vec3(
                (4.0 * 2.0 * PI / 6.0).cos(),
                (4.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[13].uv, vec2(5.0 / 6.0, 1.0));
        assert_eq!(indices[13], 13);
        assert_eq!(
            vertices[14].position,
            vec3(
                (5.0 * 2.0 * PI / 6.0).cos(),
                (5.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[14].uv, vec2(5.0 / 6.0, 0.0));
        assert_eq!(indices[14], 14);

        // Triangle 6
        assert_eq!(vertices[15].position, vec3(0.0, 0.0, 0.0));
        assert_eq!(vertices[15].uv, vec2(5.0 / 6.0, 0.0));
        assert_eq!(indices[15], 15);
        assert_eq!(
            vertices[16].position,
            vec3(
                (5.0 * 2.0 * PI / 6.0).cos(),
                (5.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[16].uv, vec2(6.0 / 6.0, 1.0));
        assert_eq!(indices[16], 16);
        assert_eq!(
            vertices[17].position,
            vec3(
                (6.0 * 2.0 * PI / 6.0).cos(),
                (6.0 * 2.0 * PI / 6.0).sin(),
                0.0
            )
        );
        assert_eq!(vertices[17].uv, vec2(6.0 / 6.0, 0.0));
        assert_eq!(indices[17], 17);
    }
}
