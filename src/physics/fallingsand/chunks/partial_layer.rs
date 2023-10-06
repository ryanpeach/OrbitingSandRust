use crate::physics::fallingsand::chunks::chunk::{interpolate_points, Chunk};
use macroquad::models::{Mesh, Vertex};
use macroquad::prelude::{vec2, vec3, Vec2, Vec3, BLUE, RED, WHITE};
use macroquad::texture::{FilterMode, Image, Texture2D};
use std::f32::consts::PI;

/// This is a chunk that represents a "full" layer.
/// It doesn't split itself in either the radial or concentric directions.
pub struct PartialLayerChunk {
    cell_radius: f32,
    start_concentric_circle_layer_relative: usize,
    start_concentric_circle_absolute: usize,
    start_radial_line: usize,
    end_radial_line: usize,
    layer_num_radial_lines: usize,
    num_concentric_circles: usize,
}

pub struct PartialLayerChunkBuilder {
    cell_radius: f32,
    start_concentric_circle_layer_relative: usize,
    start_concentric_circle_absolute: usize,
    start_radial_line: usize,
    end_radial_line: usize,
    layer_num_radial_lines: usize,
    num_concentric_circles: usize,
}

impl PartialLayerChunkBuilder {
    /// Defaults to first layer defaults
    pub fn new() -> PartialLayerChunkBuilder {
        PartialLayerChunkBuilder {
            cell_radius: 1.0,
            start_concentric_circle_layer_relative: 0,
            start_concentric_circle_absolute: 1,
            start_radial_line: 0,
            end_radial_line: 12,
            layer_num_radial_lines: 12,
            num_concentric_circles: 2,
        }
    }

    pub fn cell_radius(mut self, cell_radius: f32) -> PartialLayerChunkBuilder {
        if cell_radius <= 0.0 {
            panic!("cell_radius must be greater than 0");
        }
        self.cell_radius = cell_radius;
        self
    }

    pub fn start_concentric_circle_layer_relative(
        mut self,
        start_concentric_circle_layer_relative: usize,
    ) -> PartialLayerChunkBuilder {
        self.start_concentric_circle_layer_relative = start_concentric_circle_layer_relative;
        self
    }

    pub fn start_concentric_circle_absolute(
        mut self,
        start_concentric_circle_absolute: usize,
    ) -> PartialLayerChunkBuilder {
        self.start_concentric_circle_absolute = start_concentric_circle_absolute;
        self
    }

    pub fn start_radial_line(mut self, start_radial_line: usize) -> PartialLayerChunkBuilder {
        self.start_radial_line = start_radial_line;
        self
    }

    pub fn end_radial_line(mut self, end_radial_line: usize) -> PartialLayerChunkBuilder {
        self.end_radial_line = end_radial_line;
        self
    }

    pub fn layer_num_radial_lines(
        mut self,
        layer_num_radial_lines: usize,
    ) -> PartialLayerChunkBuilder {
        if layer_num_radial_lines == 0 {
            panic!("layer_num_radial_lines must be greater than 0");
        }
        self.layer_num_radial_lines = layer_num_radial_lines;
        self
    }

    pub fn num_concentric_circles(
        mut self,
        num_concentric_circles: usize,
    ) -> PartialLayerChunkBuilder {
        if num_concentric_circles == 0 {
            panic!("num_concentric_circles must be greater than 0");
        }
        self.num_concentric_circles = num_concentric_circles;
        self
    }

    pub fn build(self) -> PartialLayerChunk {
        if self.end_radial_line <= self.start_radial_line {
            panic!("end_radial_line must be greater than start_radial_line");
        }
        if self.end_radial_line > self.layer_num_radial_lines {
            panic!("end_radial_line must be less than or equal to layer_num_radial_lines");
        }
        PartialLayerChunk {
            cell_radius: self.cell_radius,
            start_concentric_circle_layer_relative: self.start_concentric_circle_layer_relative,
            start_concentric_circle_absolute: self.start_concentric_circle_absolute,
            start_radial_line: self.start_radial_line,
            end_radial_line: self.end_radial_line,
            layer_num_radial_lines: self.layer_num_radial_lines,
            num_concentric_circles: self.num_concentric_circles,
        }
    }
}

impl PartialLayerChunk {
    fn get_circle_vertexes(&self) -> Vec<Vec3> {
        let mut vertexes: Vec<Vec3> = Vec::new();

        let start_concentric = self.start_concentric_circle_layer_relative;
        let start_radial = self.start_radial_line;

        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let circle_separation_distance =
            (ending_r - starting_r) / (self.num_concentric_circles + 1) as f32;
        let theta = (-2.0 * PI) / self.layer_num_radial_lines as f32;

        for j in start_concentric..=self.num_concentric_circles + start_concentric {
            let diff = (j - start_concentric) as f32 * circle_separation_distance;
            let mut v_next = vec3(0.0, 0.0, 0.0);

            for k in start_radial..=self.end_radial_line {
                if j == 0 && k % 2 == 1 {
                    let angle_next = (k + 1) as f32 * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().unwrap();
                    v_next = vec3(angle_next.cos() * radius, angle_next.sin() * radius, 0.0);
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else if j == 0 && k % 2 == 0 && k != start_radial {
                    vertexes.push(v_next);
                } else {
                    let angle_point = k as f32 * theta;
                    let radius = starting_r + diff;
                    let new_coord =
                        vec3(angle_point.cos() * radius, angle_point.sin() * radius, 0.0);
                    vertexes.push(new_coord);
                }
            }
        }

        vertexes
    }

    fn get_uv_vertexes(&self) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        for j in 0..=self.num_concentric_circles {
            for k in 0..=self.get_num_radial_lines() {
                let new_vec = vec2(
                    k as f32 / self.get_num_radial_lines() as f32,
                    j as f32 / self.num_concentric_circles as f32,
                );
                vertexes.push(new_vec);
            }
        }

        vertexes
    }

    fn get_indices(&self) -> Vec<u16> {
        let mut indices = Vec::new();

        for j in 0..self.num_concentric_circles {
            for k in 0..self.get_num_radial_lines() {
                // Compute the four corners of our current grid cell
                let v0 = j * (self.get_num_radial_lines() + 1) + k; // Top-left
                let v1 = v0 + 1; // Top-right
                let v2 = v0 + (self.get_num_radial_lines() + 1) + 1; // Bottom-right
                let v3 = v0 + (self.get_num_radial_lines() + 1); // Bottom-left

                // First triangle (top-left, bottom-left, top-right)
                indices.push(v0 as u16);
                indices.push(v3 as u16);
                indices.push(v1 as u16);

                // Second triangle (top-right, bottom-left, bottom-right)
                indices.push(v1 as u16);
                indices.push(v3 as u16);
                indices.push(v2 as u16);
            }
        }

        indices
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        let positions = self.get_circle_vertexes();
        let uvs = self.get_uv_vertexes();
        let mut vertices = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            vertices.push(Vertex {
                position: positions[i],
                uv: uvs[i],
                color: WHITE,
            });
        }
        vertices
    }

    /// Right now we are just going to return a checkerboard texture
    fn get_texture(&self) -> Texture2D {
        let mut image = Image::gen_image_color(
            self.get_num_radial_lines().try_into().unwrap(),
            self.num_concentric_circles.try_into().unwrap(),
            WHITE,
        );
        let mut i = 0;
        for j in 0..self.num_concentric_circles {
            for k in 0..self.get_num_radial_lines() {
                image.set_pixel(
                    k.try_into().unwrap(),
                    j.try_into().unwrap(),
                    if i % 2 == 0 { RED } else { BLUE },
                );
                i += 1;
            }
            i += 1;
        }
        let tex = Texture2D::from_image(&image);
        tex.set_filter(FilterMode::Nearest);
        tex
    }
}

impl Chunk for PartialLayerChunk {
    fn get_positions(&self) -> Vec<Vec3> {
        self.get_circle_vertexes()
    }
    fn get_indices(&self) -> Vec<u16> {
        self.get_indices()
    }
    fn get_uvs(&self) -> Vec<Vec2> {
        self.get_uv_vertexes()
    }
    fn get_texture(&self) -> Texture2D {
        self.get_texture()
    }
    fn get_cell_radius(&self) -> f32 {
        self.cell_radius
    }
    fn get_start_radius(&self) -> f32 {
        self.start_concentric_circle_absolute as f32 * self.cell_radius
    }
    fn get_end_radius(&self) -> f32 {
        self.get_start_radius() + self.cell_radius * ((self.num_concentric_circles + 1) as f32)
    }
    fn get_num_radial_lines(&self) -> usize {
        self.end_radial_line - self.start_radial_line
    }
    fn get_num_concentric_circles(&self) -> usize {
        self.num_concentric_circles
    }
    // fn get_end_radial_theta(&self) -> f32 {
    //     let diff = (-2.0 * PI) / self.layer_num_radial_lines as f32;
    //     self.end_radial_line as f32 * diff
    // }
    // fn get_start_radial_theta(&self) -> f32 {
    //     let diff = (-2.0 * PI) / self.layer_num_radial_lines as f32;
    //     self.start_radial_line as f32 * diff
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec3_approx_eq(a: Vec3, b: Vec3, epsilon: f32) -> bool {
        (a.x - b.x).abs() < epsilon && (a.y - b.y).abs() < epsilon && (a.z - b.z).abs() < epsilon
    }

    fn vec2_approx_eq(a: Vec2, b: Vec2, epsilon: f32) -> bool {
        (a.x - b.x).abs() < epsilon && (a.y - b.y).abs() < epsilon
    }

    macro_rules! assert_approx_eq_v3 {
        ($a:expr, $b:expr) => {
            assert!(
                vec3_approx_eq($a, $b, 1e-4),
                "Vectors not approximately equal: {:?} vs {:?}",
                $a,
                $b
            )
        };
    }

    macro_rules! assert_approx_eq_v2 {
        ($a:expr, $b:expr) => {
            assert!(
                vec2_approx_eq($a, $b, 1e-4),
                "Vectors not approximately equal: {:?} vs {:?}",
                $a,
                $b
            );
        };
    }

    const FIRST_LAYER: PartialLayerChunk = PartialLayerChunk {
        cell_radius: 1.0,
        num_concentric_circles: 2,
        start_concentric_circle_layer_relative: 0,
        start_radial_line: 0,
        end_radial_line: 12,
        layer_num_radial_lines: 12,
        start_concentric_circle_absolute: 1,
    };

    #[test]
    fn test_first_layer_circle() {
        let vertices = FIRST_LAYER.get_circle_vertexes();
        assert_eq!(vertices.len(), 13 * 3);

        // The inner circle
        // every other vertex is actually an interpolation of the previous layer's num_radial_lines
        let radius = 1.0;
        let num_radial_lines = 12;
        assert_approx_eq_v3!(vertices[0], vec3(radius, 0.0, 0.0));
        assert_approx_eq_v3!(vertices[1], interpolate_points(&vertices[0], &vertices[2]));
        assert_approx_eq_v3!(
            vertices[2],
            vec3(
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(vertices[3], interpolate_points(&vertices[2], &vertices[4]));
        assert_approx_eq_v3!(
            vertices[4],
            vec3(
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(vertices[5], interpolate_points(&vertices[4], &vertices[6]));
        assert_approx_eq_v3!(
            vertices[6],
            vec3(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(vertices[7], interpolate_points(&vertices[6], &vertices[8]));
        assert_approx_eq_v3!(
            vertices[8],
            vec3(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(vertices[9], interpolate_points(&vertices[8], &vertices[10]));
        assert_approx_eq_v3!(
            vertices[10],
            vec3(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[11],
            interpolate_points(&vertices[10], &vertices[12])
        );
        assert_approx_eq_v3!(
            vertices[12],
            vec3(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );

        // The middle circle
        let radius = 2.0;
        let num_radial_lines = 12;
        assert_approx_eq_v3!(vertices[13], vec3(radius, 0.0, 0.0));
        assert_approx_eq_v3!(
            vertices[14],
            vec3(
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[15],
            vec3(
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[16],
            vec3(
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[17],
            vec3(
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[18],
            vec3(
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[19],
            vec3(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[20],
            vec3(
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[21],
            vec3(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[22],
            vec3(
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[23],
            vec3(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[24],
            vec3(
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[25],
            vec3(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );

        // The outer circle
        let radius = 3.0;
        let num_radial_lines = 12;
        assert_approx_eq_v3!(vertices[26], vec3(radius, 0.0, 0.0));
        assert_approx_eq_v3!(
            vertices[27],
            vec3(
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[28],
            vec3(
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[29],
            vec3(
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[30],
            vec3(
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[31],
            vec3(
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[32],
            vec3(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[33],
            vec3(
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[34],
            vec3(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[35],
            vec3(
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[36],
            vec3(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[37],
            vec3(
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[38],
            vec3(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
    }

    #[test]
    fn test_first_layer_uv() {
        let uvs = FIRST_LAYER.get_uv_vertexes();
        assert_eq!(uvs.len(), 13 * 3);

        // Test first layer
        let num_radial_lines = 12.0;
        assert_approx_eq_v2!(uvs[0], vec2(0.0, 0.0));
        assert_approx_eq_v2!(uvs[1], vec2(1.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[2], vec2(2.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[3], vec2(3.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[4], vec2(4.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[5], vec2(5.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[6], vec2(6.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[7], vec2(7.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[8], vec2(8.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[9], vec2(9.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[10], vec2(10.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[11], vec2(11.0 / num_radial_lines, 0.0));
        assert_approx_eq_v2!(uvs[12], vec2(12.0 / num_radial_lines, 0.0));

        // Middle layer
        let num_radial_lines = 12.0;
        let num_concentric_circles = 2.0;
        assert_approx_eq_v2!(uvs[13], vec2(0.0, 1.0 / num_concentric_circles));
        assert_approx_eq_v2!(
            uvs[14],
            vec2(1.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[15],
            vec2(2.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[16],
            vec2(3.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[17],
            vec2(4.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[18],
            vec2(5.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[19],
            vec2(6.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[20],
            vec2(7.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[21],
            vec2(8.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[22],
            vec2(9.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[23],
            vec2(10.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[24],
            vec2(11.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[25],
            vec2(12.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );

        // Outer layer
        let num_radial_lines = 12.0;
        let num_concentric_circles = 2.0;
        assert_approx_eq_v2!(uvs[26], vec2(0.0, 2.0 / num_concentric_circles));
        assert_approx_eq_v2!(
            uvs[27],
            vec2(1.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[28],
            vec2(2.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[29],
            vec2(3.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[30],
            vec2(4.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[31],
            vec2(5.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[32],
            vec2(6.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[33],
            vec2(7.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[34],
            vec2(8.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[35],
            vec2(9.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[36],
            vec2(10.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[37],
            vec2(11.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[38],
            vec2(12.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
    }

    #[test]
    fn test_first_layer_indices() {
        let indices = FIRST_LAYER.get_indices();

        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 13);
        assert_eq!(indices[2], 1);

        assert_eq!(indices[3], 1);
        assert_eq!(indices[4], 13);
        assert_eq!(indices[5], 14);

        assert_eq!(indices[6], 1);
        assert_eq!(indices[7], 14);
        assert_eq!(indices[8], 2);

        // ...

        assert_eq!(indices[indices.len() - 3], 25);
        assert_eq!(indices[indices.len() - 2], 37);
        assert_eq!(indices[indices.len() - 1], 38);
    }

    const FIRST_LAYER_PARTIAL: PartialLayerChunk = PartialLayerChunk {
        cell_radius: 1.0,
        num_concentric_circles: 1,
        start_concentric_circle_layer_relative: 1,
        start_concentric_circle_absolute: 2,
        start_radial_line: 6,
        end_radial_line: 12,
        layer_num_radial_lines: 12,
    };

    #[test]
    fn test_first_layer_circle_partial() {
        let vertices = FIRST_LAYER_PARTIAL.get_circle_vertexes();
        assert_eq!(vertices.len(), 14);

        let radius = 2.0;
        let num_radial_lines = 12;
        println!("radius: {}", radius);
        println!("num_radial_lines: {}", num_radial_lines);
        assert_approx_eq_v3!(
            vertices[0],
            vec3(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[1],
            vec3(
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[2],
            vec3(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[3],
            vec3(
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[4],
            vec3(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[5],
            vec3(
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[6],
            vec3(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );

        let radius = 3.0;
        assert_approx_eq_v3!(
            vertices[7],
            vec3(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[8],
            vec3(
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[9],
            vec3(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[10],
            vec3(
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[11],
            vec3(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[12],
            vec3(
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                0.0
            )
        );
        assert_approx_eq_v3!(
            vertices[13],
            vec3(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                0.0
            )
        )
    }

    #[test]
    fn test_first_layer_uv_partial() {
        let uvs = FIRST_LAYER_PARTIAL.get_uv_vertexes();
        assert_eq!(uvs.len(), 14);

        // Middle layer
        let num_radial_lines = 6.0;
        let num_concentric_circles = 1.0;
        assert_approx_eq_v2!(
            uvs[0],
            vec2(0.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[1],
            vec2(1.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[2],
            vec2(2.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[3],
            vec2(3.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[4],
            vec2(4.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[5],
            vec2(5.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[6],
            vec2(6.0 / num_radial_lines, 0.0 / num_concentric_circles)
        );

        assert_approx_eq_v2!(
            uvs[7],
            vec2(0.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[8],
            vec2(1.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[9],
            vec2(2.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[10],
            vec2(3.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[11],
            vec2(4.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[12],
            vec2(5.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[13],
            vec2(6.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
    }
}
