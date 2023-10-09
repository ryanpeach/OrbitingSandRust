use crate::physics::fallingsand::chunks::chunk::{interpolate_points, Chunk};
use ggez::glam::Vec2;
use ggez::graphics::{Color, Image, ImageFormat};
use ggez::Context;
use std::f32::consts::PI;

/// This is like the "skip" method but it always keeps the first and last item
/// If it is larger than the number of items, it will just return the first and last item
/// If the step is not a multiple of the number of items, it will round down to the previous multiple
fn grid_iter(start: usize, end: usize, mut step: usize) -> Vec<usize> {
    let len = end - start;
    if len == 1 {
        // Return [0]
        return vec![start];
    }
    if step >= len {
        return vec![start, end - 1];
    }
    debug_assert_ne!(step, 0, "Step should not be 0.");

    fn valid_step(len: usize, step: usize) -> bool {
        step == 1 || step == len - 1 || (len - 1) % step == 0
    }

    while !valid_step(len, step) && step > 1 {
        step -= 1;
    }

    let start_item = start;
    let end_item = end - 1;

    let mut out = Vec::new();
    out.push(start_item);
    for i in (start_item + step..end_item).step_by(step) {
        if i % step == 0 && i != 0 && i != len - 1 {
            out.push(i);
        }
    }
    out.push(end_item);
    out
}

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
        debug_assert!(cell_radius > 0.0);
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
        debug_assert_ne!(layer_num_radial_lines, 0);
        self.layer_num_radial_lines = layer_num_radial_lines;
        self
    }

    pub fn num_concentric_circles(
        mut self,
        num_concentric_circles: usize,
    ) -> PartialLayerChunkBuilder {
        debug_assert_ne!(num_concentric_circles, 0);
        self.num_concentric_circles = num_concentric_circles;
        self
    }

    pub fn build(self) -> PartialLayerChunk {
        debug_assert!(self.end_radial_line > self.start_radial_line);
        debug_assert!(self.end_radial_line <= self.layer_num_radial_lines);
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
    /// Gets the positions of the vertexes of the chunk
    /// These represent a radial grid of cells
    /// If you set skip to 1, you will get the full resolution
    /// If you set skip to 2, you will get half the resolution
    /// ...
    fn get_circle_vertexes(&self, step: usize) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric = self.start_concentric_circle_layer_relative;
        let start_radial = self.start_radial_line;

        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let circle_separation_distance =
            (ending_r - starting_r) / self.get_num_concentric_circles() as f32;
        let theta = (-2.0 * PI) / self.layer_num_radial_lines as f32;

        for j in grid_iter(
            start_concentric,
            self.get_num_concentric_circles() + start_concentric + 1,
            step,
        ) {
            let diff = (j - start_concentric) as f32 * circle_separation_distance;
            let mut v_next = Vec2::new(0.0, 0.0);

            for k in grid_iter(start_radial, self.end_radial_line + 1, step) {
                if j == 0 && k % 2 == 1 {
                    let angle_next = (k + 1) as f32 * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().unwrap();
                    v_next = Vec2::new(angle_next.cos() * radius, angle_next.sin() * radius);
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else if j == 0 && k % 2 == 0 && k != start_radial {
                    vertexes.push(v_next);
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
    fn get_outline(&self, step: usize) -> Vec<Vec2> {
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
            let mut v_next = Vec2::new(0.0, 0.0);

            for k in grid_iter(start_radial, self.end_radial_line + 1, step) {
                if j == 0 && k % 2 == 1 {
                    let angle_next = (k + 1) as f32 * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().unwrap();
                    v_next = Vec2::new(angle_next.cos() * radius, angle_next.sin() * radius);
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else if j == 0 && k % 2 == 0 && k != start_radial {
                    vertexes.push(v_next);
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

    /// Gets the UV coordinates of the vertexes of the chunk
    /// This is a more traditional square grid
    /// If you set skip to 1, you will get the full resolution
    /// If you set skip to 2, you will get half the resolution
    /// ...
    fn get_uv_vertexes(&self, step: usize) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        for j in grid_iter(0, self.get_num_concentric_circles() + 1, step) {
            for k in grid_iter(0, self.get_num_radial_lines() + 1, step) {
                let new_vec = Vec2::new(
                    k as f32 / self.get_num_radial_lines() as f32,
                    j as f32 / self.get_num_concentric_circles() as f32,
                );
                vertexes.push(new_vec);
            }
        }

        vertexes
    }

    fn get_indices(&self, step: usize) -> Vec<u32> {
        let j_iter = grid_iter(0, self.get_num_concentric_circles(), step);
        let j_count = j_iter.len();
        let k_iter = grid_iter(0, self.get_num_radial_lines(), step);
        let k_count = k_iter.len();
        let mut indices = Vec::with_capacity(j_count * k_count * 6);
        for j in 0..j_count {
            for k in 0..k_count {
                // Compute the four corners of our current grid cell
                let v0 = j * (self.get_num_radial_lines() + 1) + k; // Top-left
                let v1 = v0 + 1; // Top-right
                let v2 = v0 + (self.get_num_radial_lines() + 1) + 1; // Bottom-right
                let v3 = v0 + (self.get_num_radial_lines() + 1); // Bottom-left

                // First triangle (top-left, bottom-left, top-right)
                indices.push(v0 as u32);
                indices.push(v3 as u32);
                indices.push(v1 as u32);

                // Second triangle (top-right, bottom-left, bottom-right)
                indices.push(v1 as u32);
                indices.push(v3 as u32);
                indices.push(v2 as u32);
            }
        }

        indices
    }

    /// Right now we are just going to return a checkerboard texture
    fn get_texture(&self, ctx: &mut Context, step: usize) -> Image {
        let j_iter = grid_iter(0, self.get_num_concentric_circles() + 1, step);
        let j_count = j_iter.len();
        let k_iter = grid_iter(0, self.get_num_radial_lines() + 1, step);
        let k_count = k_iter.len();
        let mut pixels: Vec<u8> = Vec::with_capacity(j_count * k_count * 4);
        let mut i = 0;
        for _ in 0..j_count {
            for _ in 0..k_count {
                let color = if i % 2 == 0 {
                    Color::YELLOW
                } else {
                    Color::BLUE
                };
                let rgba = color.to_rgba();
                pixels.push(rgba.0);
                pixels.push(rgba.1);
                pixels.push(rgba.2);
                pixels.push(rgba.3);
                i += 1;
            }
        }
        Image::from_pixels(
            ctx,
            &pixels[..],
            ImageFormat::Rgba8Unorm,
            k_count as u32,
            j_count as u32,
        )
    }
}

impl Chunk for PartialLayerChunk {
    fn get_outline(&self, res: u16) -> Vec<Vec2> {
        self.get_outline(2usize.pow(res.into()))
    }
    fn get_positions(&self, res: u16) -> Vec<Vec2> {
        self.get_circle_vertexes(2usize.pow(res.into()))
    }
    fn get_indices(&self, res: u16) -> Vec<u32> {
        self.get_indices(2usize.pow(res.into()))
    }
    fn get_uvs(&self, res: u16) -> Vec<Vec2> {
        self.get_uv_vertexes(2usize.pow(res.into()))
    }
    fn get_texture(&self, ctx: &mut Context, res: u16) -> Image {
        self.get_texture(ctx, 2usize.pow(res.into()))
    }
    fn get_cell_radius(&self) -> f32 {
        self.cell_radius
    }
    fn get_start_radius(&self) -> f32 {
        self.start_concentric_circle_absolute as f32 * self.cell_radius
            + self.start_concentric_circle_layer_relative as f32 * self.cell_radius
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
    fn get_end_concentric_circle_relative(&self) -> usize {
        self.start_concentric_circle_layer_relative + self.num_concentric_circles
    }
    fn get_end_radial_line(&self) -> usize {
        self.end_radial_line
    }
    fn get_start_radial_line(&self) -> usize {
        self.start_radial_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_element() {
        let v: Vec<_> = grid_iter(0, 1, 16);
        assert_eq!(v, vec![0]);
    }

    #[test]
    fn test_two_elements() {
        let v: Vec<_> = grid_iter(0, 2, 16);
        assert_eq!(v, vec![0, 1]);
    }

    #[test]
    fn test_basic() {
        let v: Vec<_> = grid_iter(0, 11, 2);
        assert_eq!(v, vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_step_one() {
        let v: Vec<_> = grid_iter(0, 11, 1);
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    /// At a large step size, we should just get the first and last elements
    #[test]
    fn test_large_step() {
        let v: Vec<_> = grid_iter(0, 10, 20);
        assert_eq!(v, vec![0, 9]);
    }

    /// In this case there is no "middle" element
    /// Two could either produce 0, 1, 3 or 0, 2, 3 both of
    /// which are invalid because they don't have constant spacing
    /// So we round down to 1
    #[test]
    fn test_weird_number_4_by_2() {
        let v: Vec<_> = grid_iter(0, 4, 2);
        assert_eq!(v, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_basic_5() {
        let v: Vec<_> = grid_iter(0, 5, 2);
        assert_eq!(v, vec![0, 2, 4]);
    }

    #[test]
    fn test_weird_6() {
        let v: Vec<_> = grid_iter(0, 6, 3);
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5]);
    }

    /// In this case, because three doesnt work, we automatically round down to 2
    #[test]
    fn test_round_7() {
        let v: Vec<_> = grid_iter(0, 7, 3);
        assert_eq!(v, vec![0, 3, 6]);
    }

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
        let vertices = FIRST_LAYER.get_circle_vertexes(1);
        assert_eq!(vertices.len(), 13 * 3);

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

        // The middle circle
        let radius = 2.0;
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

        // The outer circle
        let radius = 3.0;
        let num_radial_lines = 12;
        assert_approx_eq_v2!(vertices[26], Vec2::new(radius, 0.0));
        assert_approx_eq_v2!(
            vertices[27],
            Vec2::new(
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -1.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[28],
            Vec2::new(
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -2.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[29],
            Vec2::new(
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -3.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[30],
            Vec2::new(
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -4.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[31],
            Vec2::new(
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -5.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[32],
            Vec2::new(
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -6.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[33],
            Vec2::new(
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -7.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[34],
            Vec2::new(
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -8.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[35],
            Vec2::new(
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -9.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[36],
            Vec2::new(
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -10.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[37],
            Vec2::new(
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -11.0 / num_radial_lines as f32).sin(),
            )
        );
        assert_approx_eq_v2!(
            vertices[38],
            Vec2::new(
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).cos(),
                radius * (2.0 * PI * -12.0 / num_radial_lines as f32).sin(),
            )
        );
    }

    #[test]
    fn test_first_layer_uv() {
        let uvs = FIRST_LAYER.get_uv_vertexes(1);
        assert_eq!(uvs.len(), 13 * 3);

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

        // Middle layer
        let num_radial_lines = 12.0;
        let num_concentric_circles = 2.0;
        assert_approx_eq_v2!(uvs[13], Vec2::new(0.0, 1.0 / num_concentric_circles));
        assert_approx_eq_v2!(
            uvs[14],
            Vec2::new(1.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[15],
            Vec2::new(2.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[16],
            Vec2::new(3.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[17],
            Vec2::new(4.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[18],
            Vec2::new(5.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[19],
            Vec2::new(6.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[20],
            Vec2::new(7.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[21],
            Vec2::new(8.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[22],
            Vec2::new(9.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[23],
            Vec2::new(10.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[24],
            Vec2::new(11.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[25],
            Vec2::new(12.0 / num_radial_lines, 1.0 / num_concentric_circles)
        );

        // Outer layer
        let num_radial_lines = 12.0;
        let num_concentric_circles = 2.0;
        assert_approx_eq_v2!(uvs[26], Vec2::new(0.0, 2.0 / num_concentric_circles));
        assert_approx_eq_v2!(
            uvs[27],
            Vec2::new(1.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[28],
            Vec2::new(2.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[29],
            Vec2::new(3.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[30],
            Vec2::new(4.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[31],
            Vec2::new(5.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[32],
            Vec2::new(6.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[33],
            Vec2::new(7.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[34],
            Vec2::new(8.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[35],
            Vec2::new(9.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[36],
            Vec2::new(10.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[37],
            Vec2::new(11.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
        assert_approx_eq_v2!(
            uvs[38],
            Vec2::new(12.0 / num_radial_lines, 2.0 / num_concentric_circles)
        );
    }

    #[test]
    fn test_first_layer_indices() {
        let indices = FIRST_LAYER.get_indices(1);
        assert_eq!(indices.len(), 12 * 2 * 6);

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

        // The second concentric circle
        for i in 13..25u32 {
            assert_eq!(indices[j], i, "i: {}", i);
            assert_eq!(indices[j + 1], i + 13u32, "i: {}", i);
            assert_eq!(indices[j + 2], i + 1u32, "i: {}", i);
            assert_eq!(indices[j + 3], i + 1u32, "i: {}", i);
            assert_eq!(indices[j + 4], i + 13u32, "i: {}", i);
            assert_eq!(indices[j + 5], i + 14u32, "i: {}", i);
            j += 6;
        }
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
        let vertices = FIRST_LAYER_PARTIAL.get_circle_vertexes(1);
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
        let uvs = FIRST_LAYER_PARTIAL.get_uv_vertexes(1);
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