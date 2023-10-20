use crate::physics::fallingsand::coordinates::chunk_coords::ChunkCoords;
use crate::physics::fallingsand::util::functions::interpolate_points;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use ggez::glam::Vec2;
use ggez::graphics::{Color, Rect};

use std::f32::consts::PI;

/// This is a chunk that represents a "full" layer.
/// It doesn't split itself in either the radial or concentric directions.
#[derive(Debug, Clone, Copy, Default)]
pub struct PartialLayerChunkCoords {
    cell_radius: f32,
    chunk_idx: ChunkIjkVector,
    start_concentric_circle_layer_relative: usize,
    start_concentric_circle_absolute: usize,
    start_radial_line: usize,
    end_radial_line: usize,
    layer_num_radial_lines: usize,
    num_concentric_circles: usize,
}

pub struct PartialLayerChunkCoordsBuilder {
    cell_radius: f32,
    chunk_idx: ChunkIjkVector,
    start_concentric_circle_layer_relative: usize,
    start_concentric_circle_absolute: usize,
    start_radial_line: usize,
    end_radial_line: usize,
    layer_num_radial_lines: usize,
    num_concentric_circles: usize,
}

impl Default for PartialLayerChunkCoordsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialLayerChunkCoordsBuilder {
    /// Defaults to first layer defaults
    pub fn new() -> PartialLayerChunkCoordsBuilder {
        PartialLayerChunkCoordsBuilder {
            cell_radius: 1.0,
            chunk_idx: ChunkIjkVector::ZERO,
            start_concentric_circle_layer_relative: 0,
            start_concentric_circle_absolute: 0,
            start_radial_line: 0,
            end_radial_line: 0,
            layer_num_radial_lines: 0,
            num_concentric_circles: 0,
        }
    }

    pub fn cell_radius(mut self, cell_radius: f32) -> PartialLayerChunkCoordsBuilder {
        debug_assert!(cell_radius > 0.0);
        self.cell_radius = cell_radius;
        self
    }

    pub fn start_concentric_circle_layer_relative(
        mut self,
        start_concentric_circle_layer_relative: usize,
    ) -> PartialLayerChunkCoordsBuilder {
        self.start_concentric_circle_layer_relative = start_concentric_circle_layer_relative;
        self
    }

    pub fn start_concentric_circle_absolute(
        mut self,
        start_concentric_circle_absolute: usize,
    ) -> PartialLayerChunkCoordsBuilder {
        self.start_concentric_circle_absolute = start_concentric_circle_absolute;
        self
    }

    pub fn start_radial_line(mut self, start_radial_line: usize) -> PartialLayerChunkCoordsBuilder {
        self.start_radial_line = start_radial_line;
        self
    }

    pub fn end_radial_line(mut self, end_radial_line: usize) -> PartialLayerChunkCoordsBuilder {
        self.end_radial_line = end_radial_line;
        self
    }

    pub fn chunk_idx(mut self, chunk_idx: ChunkIjkVector) -> PartialLayerChunkCoordsBuilder {
        self.chunk_idx = chunk_idx;
        self
    }

    pub fn layer_num_radial_lines(
        mut self,
        layer_num_radial_lines: usize,
    ) -> PartialLayerChunkCoordsBuilder {
        debug_assert_ne!(layer_num_radial_lines, 0);
        self.layer_num_radial_lines = layer_num_radial_lines;
        self
    }

    pub fn num_concentric_circles(
        mut self,
        num_concentric_circles: usize,
    ) -> PartialLayerChunkCoordsBuilder {
        debug_assert_ne!(num_concentric_circles, 0);
        self.num_concentric_circles = num_concentric_circles;
        self
    }

    pub fn build(self) -> PartialLayerChunkCoords {
        debug_assert!(self.end_radial_line > self.start_radial_line);
        debug_assert!(self.end_radial_line <= self.layer_num_radial_lines);
        debug_assert_ne!(self.num_concentric_circles, 0);
        debug_assert_ne!(self.start_concentric_circle_absolute, 0);
        debug_assert_ne!(self.layer_num_radial_lines, 0);
        debug_assert_ne!(self.end_radial_line, 0);
        PartialLayerChunkCoords {
            cell_radius: self.cell_radius,
            start_concentric_circle_layer_relative: self.start_concentric_circle_layer_relative,
            start_concentric_circle_absolute: self.start_concentric_circle_absolute,
            start_radial_line: self.start_radial_line,
            end_radial_line: self.end_radial_line,
            chunk_idx: self.chunk_idx,
            layer_num_radial_lines: self.layer_num_radial_lines,
            num_concentric_circles: self.num_concentric_circles,
        }
    }
}

impl PartialLayerChunkCoords {
    /// Gets the positions of the vertexes of the chunk
    /// These represent a radial grid of cells
    /// If you set skip to 1, you will get the full resolution
    /// If you set skip to 2, you will get half the resolution
    /// ...
    fn get_circle_vertexes(&self) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric = self.start_concentric_circle_layer_relative;
        let start_radial = self.start_radial_line;

        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let circle_separation_distance =
            (ending_r - starting_r) / self.get_num_concentric_circles() as f32;
        let theta = (-2.0 * PI) / self.layer_num_radial_lines as f32;

        for j in [
            start_concentric,
            self.get_num_concentric_circles() + start_concentric,
        ] {
            let diff = (j - start_concentric) as f32 * circle_separation_distance;

            for k in start_radial..(self.end_radial_line + 1) {
                if j == 0 && k % 2 == 1 {
                    let angle_next = (k + 1) as f32 * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().unwrap();
                    let v_next = Vec2::new(angle_next.cos() * radius, angle_next.sin() * radius);
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else {
                    let angle_point = k as f32 * theta;
                    let radius = starting_r + diff;
                    let new_coord =
                        Vec2::new(angle_point.cos() * radius, angle_point.sin() * radius);
                    vertexes.push(new_coord);
                }
            }
        }

        vertexes
    }

    /// Similar to get_circle_vertexes, but the j index just iterates on the 0th and last element
    fn get_outline(&self) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric = self.start_concentric_circle_layer_relative;
        let start_radial = self.start_radial_line;

        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let circle_separation_distance =
            (ending_r - starting_r) / self.get_num_concentric_circles() as f32;
        let theta = (-2.0 * PI) / self.layer_num_radial_lines as f32;

        for j in [
            start_concentric,
            self.get_num_concentric_circles() + start_concentric,
        ] {
            let diff = (j - start_concentric) as f32 * circle_separation_distance;

            // Reverse if we are on the last element because we are going around the circle
            // This box method was the only way to make Range == Rev<Range> in type, very annoying.
            let iter: Box<dyn Iterator<Item = _>> = if j != start_concentric {
                Box::new((start_radial..self.end_radial_line + 1).rev())
            } else {
                Box::new(start_radial..self.end_radial_line + 1)
            };

            for k in iter {
                if j == 0 && k % 2 == 1 {
                    let angle_next = (k + 1) as f32 * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().unwrap();
                    let v_next = Vec2::new(angle_next.cos() * radius, angle_next.sin() * radius);
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else {
                    let angle_point = k as f32 * theta;
                    let radius = starting_r + diff;
                    let new_coord =
                        Vec2::new(angle_point.cos() * radius, angle_point.sin() * radius);
                    vertexes.push(new_coord);
                }
            }
        }

        vertexes
    }

    /// Gets the min and max positions in raw x, y of the chunk
    fn get_bounding_box(&self) -> Rect {
        let outline = self.get_outline();
        let all_x = outline.iter().map(|v| v.x);
        let all_y = outline.iter().map(|v| v.y);
        let min_x = all_x.clone().fold(f32::INFINITY, f32::min);
        let max_x = all_x.fold(f32::NEG_INFINITY, f32::max);
        let min_y = all_y.clone().fold(f32::INFINITY, f32::min);
        let max_y = all_y.fold(f32::NEG_INFINITY, f32::max);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

impl ChunkCoords for PartialLayerChunkCoords {
    fn get_outline(&self) -> Vec<Vec2> {
        self.get_outline()
    }
    fn get_positions(&self) -> Vec<Vec2> {
        self.get_circle_vertexes()
    }
    fn get_cell_radius(&self) -> f32 {
        self.cell_radius
    }
    fn get_start_radius(&self) -> f32 {
        self.start_concentric_circle_absolute as f32 * self.cell_radius
    }
    fn get_end_radius(&self) -> f32 {
        self.get_start_radius() + self.cell_radius * (self.num_concentric_circles as f32)
    }
    fn get_num_radial_lines(&self) -> usize {
        self.end_radial_line - self.start_radial_line
    }
    fn get_num_concentric_circles(&self) -> usize {
        self.num_concentric_circles
    }
    fn get_end_radial_theta(&self) -> f32 {
        let diff = (2.0 * PI) / self.layer_num_radial_lines as f32;
        self.end_radial_line as f32 * diff
    }
    fn get_start_radial_theta(&self) -> f32 {
        let diff = (2.0 * PI) / self.layer_num_radial_lines as f32;
        self.start_radial_line as f32 * diff
    }
    fn get_start_concentric_circle_layer_relative(&self) -> usize {
        self.start_concentric_circle_layer_relative
    }
    fn get_start_concentric_circle_absolute(&self) -> usize {
        self.start_concentric_circle_absolute
    }
    fn get_end_concentric_circle_absolute(&self) -> usize {
        self.start_concentric_circle_absolute + self.num_concentric_circles
    }
    fn get_end_concentric_circle_layer_relative(&self) -> usize {
        self.start_concentric_circle_layer_relative + self.num_concentric_circles
    }
    fn get_end_radial_line(&self) -> usize {
        self.end_radial_line
    }
    fn get_start_radial_line(&self) -> usize {
        self.start_radial_line
    }
    fn get_layer_num(&self) -> usize {
        self.chunk_idx.i
    }
    fn get_chunk_idx(&self) -> ChunkIjkVector {
        self.chunk_idx
    }
    fn get_bounding_box(&self) -> Rect {
        self.get_bounding_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec2_approx_eq(a: Vec2, b: Vec2, epsilon: f32) -> bool {
        (a.x - b.x).abs() < epsilon && (a.y - b.y).abs() < epsilon
    }

    macro_rules! assert_approx_eq_v2 {
        ($a:expr, $b:expr) => {
            assert!(
                vec2_approx_eq($a, $b, 1e-4),
                "Vectors not approximately equal: {:?} vs {:?}",
                $a,
                $b
            )
        };
    }

    mod full_layer {
        use super::*;

        const FIRST_LAYER: PartialLayerChunkCoords = PartialLayerChunkCoords {
            cell_radius: 1.0,
            num_concentric_circles: 2,
            chunk_idx: ChunkIjkVector { i: 1, j: 0, k: 0 },
            start_concentric_circle_layer_relative: 0,
            start_radial_line: 0,
            end_radial_line: 12,
            layer_num_radial_lines: 12,
            start_concentric_circle_absolute: 1,
        };

        #[test]
        fn test_first_layer_circle() {
            let vertices = FIRST_LAYER.get_circle_vertexes();
            assert_eq!(vertices.len(), 13 * 2);

            // The inner circle
            // every other vertex is actually an interpolation of the previous layer's num_radial_lines
            let radius = 1.0;
            let num_radial_lines = 12;
            assert_approx_eq_v2!(vertices[0], Vec2::new(radius, 0.0));
            assert_approx_eq_v2!(vertices[1], interpolate_points(&vertices[0], &vertices[2]));
            assert_approx_eq_v2!(
                vertices[2],
                Vec2::new(
                    radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(vertices[3], interpolate_points(&vertices[2], &vertices[4]));
            assert_approx_eq_v2!(
                vertices[4],
                Vec2::new(
                    radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(vertices[5], interpolate_points(&vertices[4], &vertices[6]));
            assert_approx_eq_v2!(
                vertices[6],
                Vec2::new(
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(vertices[7], interpolate_points(&vertices[6], &vertices[8]));
            assert_approx_eq_v2!(
                vertices[8],
                Vec2::new(
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(vertices[9], interpolate_points(&vertices[8], &vertices[10]));
            assert_approx_eq_v2!(
                vertices[10],
                Vec2::new(
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[11],
                interpolate_points(&vertices[10], &vertices[12])
            );
            assert_approx_eq_v2!(
                vertices[12],
                Vec2::new(
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                )
            );

            // The outer circle
            let radius = 3.0;
            let num_radial_lines = 12;
            assert_approx_eq_v2!(vertices[13], Vec2::new(radius, 0.0));
            assert_approx_eq_v2!(
                vertices[14],
                Vec2::new(
                    radius * (2.0 * PI * -1.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -1.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[15],
                Vec2::new(
                    radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[16],
                Vec2::new(
                    radius * (2.0 * PI * -3.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -3.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[17],
                Vec2::new(
                    radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[18],
                Vec2::new(
                    radius * (2.0 * PI * -5.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -5.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[19],
                Vec2::new(
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[20],
                Vec2::new(
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[21],
                Vec2::new(
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[22],
                Vec2::new(
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[23],
                Vec2::new(
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[24],
                Vec2::new(
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[25],
                Vec2::new(
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                )
            );
        }

        #[test]
        fn test_first_layer_uv() {
            let uvs = FIRST_LAYER.get_uv_vertexes();
            assert_eq!(uvs.len(), 13 * 2);

            // Test first layer
            let num_radial_lines = 12.0;
            assert_approx_eq_v2!(uvs[0], Vec2::new(0.0, 0.0));
            assert_approx_eq_v2!(uvs[1], Vec2::new(1.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[2], Vec2::new(2.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[3], Vec2::new(3.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[4], Vec2::new(4.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[5], Vec2::new(5.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[6], Vec2::new(6.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[7], Vec2::new(7.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[8], Vec2::new(8.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[9], Vec2::new(9.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[10], Vec2::new(10.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[11], Vec2::new(11.0 / num_radial_lines, 0.0));
            assert_approx_eq_v2!(uvs[12], Vec2::new(12.0 / num_radial_lines, 0.0));

            // Outer layer
            let num_radial_lines = 12.0;
            let num_concentric_circles = 2.0;
            assert_approx_eq_v2!(uvs[13], Vec2::new(0.0, 2.0 / num_concentric_circles));
            assert_approx_eq_v2!(
                uvs[14],
                Vec2::new(1.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[15],
                Vec2::new(2.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[16],
                Vec2::new(3.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[17],
                Vec2::new(4.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[18],
                Vec2::new(5.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[19],
                Vec2::new(6.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[20],
                Vec2::new(7.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[21],
                Vec2::new(8.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[22],
                Vec2::new(9.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[23],
                Vec2::new(10.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[24],
                Vec2::new(11.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[25],
                Vec2::new(12.0 / num_radial_lines, 2.0 / num_concentric_circles)
            );
        }

        #[test]
        fn test_first_layer_indices() {
            let indices = FIRST_LAYER.get_indices();
            assert_eq!(indices.len(), 12 * 6);

            // The first concentric circle
            let mut j = 0;
            for i in 0..12u32 {
                assert_eq!(indices[j], i, "i: {}", i);
                assert_eq!(indices[j + 1], i + 13u32, "i: {}", i);
                assert_eq!(indices[j + 2], i + 1u32, "i: {}", i);
                assert_eq!(indices[j + 3], i + 1u32, "i: {}", i);
                assert_eq!(indices[j + 4], i + 13u32, "i: {}", i);
                assert_eq!(indices[j + 5], i + 14u32, "i: {}", i);
                j += 6;
            }
        }

        #[test]
        fn test_first_layer_bounding_box() {
            let bb = FIRST_LAYER.get_bounding_box();
            assert_eq!(bb.x, -3.0);
            assert_eq!(bb.y, -3.0);
            assert_eq!(bb.w, 6.0);
            assert_eq!(bb.h, 6.0);
        }
    }

    mod partial_layer {
        use super::*;

        const FIRST_LAYER_PARTIAL: PartialLayerChunkCoords = PartialLayerChunkCoords {
            cell_radius: 1.0,
            num_concentric_circles: 1,
            chunk_idx: ChunkIjkVector { i: 1, j: 0, k: 0 },
            start_concentric_circle_layer_relative: 1,
            start_concentric_circle_absolute: 3,
            start_radial_line: 6,
            end_radial_line: 12,
            layer_num_radial_lines: 12,
        };

        #[test]
        fn test_first_layer_circle_partial() {
            let vertices = FIRST_LAYER_PARTIAL.get_circle_vertexes();
            assert_eq!(vertices.len(), 14);

            let radius = 3.0;
            let num_radial_lines = 12;
            println!("radius: {}", radius);
            println!("num_radial_lines: {}", num_radial_lines);
            assert_approx_eq_v2!(
                vertices[0],
                Vec2::new(
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[1],
                Vec2::new(
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[2],
                Vec2::new(
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[3],
                Vec2::new(
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[4],
                Vec2::new(
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[5],
                Vec2::new(
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[6],
                Vec2::new(
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
                )
            );

            let radius = 4.0;
            assert_approx_eq_v2!(
                vertices[7],
                Vec2::new(
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[8],
                Vec2::new(
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[9],
                Vec2::new(
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[10],
                Vec2::new(
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[11],
                Vec2::new(
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[12],
                Vec2::new(
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
                )
            );
            assert_approx_eq_v2!(
                vertices[13],
                Vec2::new(
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                    radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
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
                Vec2::new(0.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[1],
                Vec2::new(1.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[2],
                Vec2::new(2.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[3],
                Vec2::new(3.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[4],
                Vec2::new(4.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[5],
                Vec2::new(5.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[6],
                Vec2::new(6.0 / num_radial_lines, 0.0 / num_concentric_circles)
            );

            assert_approx_eq_v2!(
                uvs[7],
                Vec2::new(0.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[8],
                Vec2::new(1.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[9],
                Vec2::new(2.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[10],
                Vec2::new(3.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[11],
                Vec2::new(4.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[12],
                Vec2::new(5.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
            assert_approx_eq_v2!(
                uvs[13],
                Vec2::new(6.0 / num_radial_lines, 1.0 / num_concentric_circles)
            );
        }
    }
}
