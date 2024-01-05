use crate::physics::fallingsand::util::functions::interpolate_points;
use crate::physics::fallingsand::util::image::RawImage;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::{ChunkIjkVector, IjkVector, JkVector};
use crate::physics::util::vectors::RelXyPoint;
use ggez::glam::Vec2;
use ggez::graphics::{Color, MeshBuilder, Rect, Vertex};

use std::f32::consts::PI;

/// The optimal way of drawing the vertexes is just to draw the radial lines.
/// Because the texture will map along the column perfectly.
/// However, if you want to see the whole grid you can use the grid vertexes.
#[derive(Debug, Clone, Copy)]
pub enum VertexMode {
    Lines,
    Grid,
}

/// This is a chunk that represents a "full" layer.
/// It doesn't split itself in either the radial or concentric directions.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkCoords {
    width: f32,
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

    pub fn build(self) -> ChunkCoords {
        debug_assert!(self.end_radial_line > self.start_radial_line);
        debug_assert!(self.end_radial_line <= self.layer_num_radial_lines);
        debug_assert_ne!(self.num_concentric_circles, 0);
        debug_assert_ne!(self.layer_num_radial_lines, 0);
        debug_assert_ne!(self.end_radial_line, 0);
        ChunkCoords {
            width: self.cell_radius,
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

impl ChunkCoords {
    pub fn get_positions(&self, mode: VertexMode) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric = self.start_concentric_circle_layer_relative;
        let start_radial = self.start_radial_line;

        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let circle_separation_distance =
            (ending_r - starting_r) / self.get_num_concentric_circles() as f32;
        let theta = (-2.0 * PI) / self.layer_num_radial_lines as f32;

        let concentric_range = match mode {
            VertexMode::Lines => vec![
                start_concentric,
                self.get_num_concentric_circles() + start_concentric,
            ],
            VertexMode::Grid => (start_concentric
                ..(self.get_num_concentric_circles() + start_concentric + 1))
                .collect::<Vec<_>>(),
        };

        for j in concentric_range {
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
    pub fn get_outline(&self) -> Vec<Vec2> {
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
    pub fn get_bounding_box(&self) -> Rect {
        let outline = self.get_outline();
        let all_x = outline.iter().map(|v| v.x);
        let all_y = outline.iter().map(|v| v.y);
        let min_x = all_x.clone().fold(f32::INFINITY, f32::min);
        let max_x = all_x.fold(f32::NEG_INFINITY, f32::max);
        let min_y = all_y.clone().fold(f32::INFINITY, f32::min);
        let max_y = all_y.fold(f32::NEG_INFINITY, f32::max);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Gets the UV coordinates of the vertexes of the chunk
    /// This is a more traditional square grid
    /// If you set skip to 1, you will get the full resolution
    /// If you set skip to 2, you will get half the resolution
    /// ...
    pub fn get_uvs(&self, mode: VertexMode) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let concentric_range = match mode {
            VertexMode::Lines => vec![0, self.get_num_concentric_circles()],
            VertexMode::Grid => (0..(self.get_num_concentric_circles() + 1)).collect::<Vec<_>>(),
        };

        for j in concentric_range {
            for k in 0..(self.get_num_radial_lines() + 1) {
                let new_vec = Vec2::new(
                    k as f32 / self.get_num_radial_lines() as f32,
                    j as f32 / self.get_num_concentric_circles() as f32,
                );
                vertexes.push(new_vec);
            }
        }

        vertexes
    }

    pub fn get_indices(&self, mode: VertexMode) -> Vec<u32> {
        let j_count = match mode {
            VertexMode::Lines => 2,
            VertexMode::Grid => self.get_num_concentric_circles() + 1,
        };
        let k_iter = 0..(self.get_num_radial_lines() + 1);
        let k_count = k_iter.len();
        let mut indices = Vec::with_capacity(j_count * k_count * 6);
        for j in 0..j_count - 1 {
            for k in 0..k_count - 1 {
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
    fn get_texture(&self, _step: usize) -> RawImage {
        let j_count = self.get_num_concentric_circles();
        let k_count = self.get_num_radial_lines();
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
            i += 1;
        }
        RawImage {
            pixels,
            bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                k_count as f32,
                j_count as f32,
            ),
        }
    }
}

impl ChunkCoords {
    pub fn total_size(&self) -> usize {
        self.get_num_radial_lines() * self.get_num_concentric_circles()
    }
    pub fn get_cell_width(&self) -> f32 {
        self.width
    }
    pub fn get_start_radius(&self) -> f32 {
        self.start_concentric_circle_absolute as f32 * self.width
    }
    pub fn get_end_radius(&self) -> f32 {
        self.get_start_radius() + self.width * (self.num_concentric_circles as f32)
    }
    pub fn get_num_radial_lines(&self) -> usize {
        self.end_radial_line - self.start_radial_line
    }
    pub fn get_num_concentric_circles(&self) -> usize {
        self.num_concentric_circles
    }
    pub fn get_end_radial_theta(&self) -> f32 {
        let diff = (2.0 * PI) / self.layer_num_radial_lines as f32;
        self.end_radial_line as f32 * diff
    }
    pub fn get_start_radial_theta(&self) -> f32 {
        let diff = (2.0 * PI) / self.layer_num_radial_lines as f32;
        self.start_radial_line as f32 * diff
    }
    pub fn get_start_concentric_circle_layer_relative(&self) -> usize {
        self.start_concentric_circle_layer_relative
    }
    pub fn get_start_concentric_circle_absolute(&self) -> usize {
        self.start_concentric_circle_absolute
    }
    pub fn get_end_concentric_circle_absolute(&self) -> usize {
        self.start_concentric_circle_absolute + self.num_concentric_circles
    }
    pub fn get_end_concentric_circle_layer_relative(&self) -> usize {
        self.start_concentric_circle_layer_relative + self.num_concentric_circles
    }
    pub fn get_end_radial_line(&self) -> usize {
        self.end_radial_line
    }
    pub fn get_start_radial_line(&self) -> usize {
        self.start_radial_line
    }
    pub fn get_layer_num(&self) -> usize {
        self.chunk_idx.i
    }
    pub fn get_chunk_idx(&self) -> ChunkIjkVector {
        self.chunk_idx
    }

    /* Positions in the chunk */
    /// Checks to see if an absolute position around the circle is in the chunk
    pub fn contains(&self, idx: IjkVector) -> bool {
        idx.i == self.get_layer_num()
            && idx.j >= self.get_start_radial_line()
            && idx.j < self.get_end_radial_line()
            && idx.k >= self.get_start_concentric_circle_absolute()
            && idx.k < self.get_end_concentric_circle_absolute()
    }
    /// Converts a coordinate from anywhere on the circle, assuming it is in the chunk
    /// to a coordinate inside the grid of this chunk
    pub fn get_internal_coord_from_external_coord(&self, external_coord: IjkVector) -> JkVector {
        debug_assert!(self.contains(external_coord));
        JkVector {
            j: external_coord.j - self.get_start_radial_line(),
            k: external_coord.k - self.get_start_concentric_circle_absolute(),
        }
    }
    /// Converts a coordinate from inside this chunk to a coordinate on the circle
    pub fn get_external_coord_from_internal_coord(&self, internal_coord: JkVector) -> IjkVector {
        debug_assert!(internal_coord.j < self.get_num_radial_lines());
        debug_assert!(internal_coord.k < self.get_num_concentric_circles());
        IjkVector {
            i: self.get_layer_num(),
            j: internal_coord.j + self.get_start_radial_line(),
            k: internal_coord.k + self.get_start_concentric_circle_absolute(),
        }
    }

    /* Convienience Functions */
    pub fn get_vertices(&self, mode: VertexMode) -> Vec<Vertex> {
        let positions = self.get_positions(mode);
        let uvs = self.get_uvs(mode);
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
    pub fn calc_chunk_outline(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let outline = self.get_outline();
        let _ = mb.line(&outline, 1.0, Color::RED);
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
    pub fn calc_chunk_meshdata(&self) -> OwnedMeshData {
        let indices = self.get_indices(VertexMode::Lines);
        let vertices: Vec<Vertex> = self.get_vertices(VertexMode::Lines);
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
    pub fn calc_chunk_triangle_wireframe(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let indices = self.get_indices(VertexMode::Grid);
        let vertices: Vec<Vertex> = self.get_vertices(VertexMode::Grid);
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

    /// Converts a position relative to the origin of the circle to a cell index
    /// Returns an Err if the position is not on the circle
    pub fn rel_pos_to_cell_idx(&self, xy_coord: RelXyPoint) -> Result<IjkVector, String> {
        let norm_vertex_coord = (xy_coord.0.x * xy_coord.0.x + xy_coord.0.y * xy_coord.0.y).sqrt();
        let start_concentric_circle = self.get_start_concentric_circle_layer_relative();
        let end_concentric_circle = self.get_end_concentric_circle_layer_relative();
        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let num_concentric_circles = self.get_num_concentric_circles();
        let num_radial_lines = self.get_num_radial_lines();
        let start_radial_line = self.get_start_radial_line();
        let end_radial_line = self.get_end_radial_line();
        let start_radial_theta = self.get_start_radial_theta();
        let end_radial_theta = self.get_end_radial_theta();

        // Get the concentric circle we are on
        let circle_separation_distance = (ending_r - starting_r) / num_concentric_circles as f32;

        // Calculate 'j' directly without the while loop
        let j_rel =
            ((norm_vertex_coord - starting_r) / circle_separation_distance).floor() as usize;
        let j = j_rel.min(end_concentric_circle - 1) + start_concentric_circle;

        // Get the radial line to the left of the vertex
        let angle = (xy_coord.0.y.atan2(xy_coord.0.x) + -2.0 * PI) % (2.0 * PI);
        let theta = -(end_radial_theta - start_radial_theta) / num_radial_lines as f32;

        // Calculate 'k' directly without the while loop
        let k_rel = (angle / theta).floor() as usize;
        let k = k_rel.min(end_radial_line - 1);

        // Check to see if the vertex is in the chunk
        if j < start_concentric_circle && j >= end_concentric_circle {
            return Err(format!(
                "Vertex j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                xy_coord,
                self.get_chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            ));
        }
        if k < start_radial_line && k >= end_radial_line {
            return Err(format!(
                "Vertex k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                xy_coord,
                self.get_chunk_idx(),
                start_radial_line,
                end_radial_line,
            ));
        }
        Ok(IjkVector {
            i: self.get_layer_num(),
            j,
            k,
        })
    }

    /// Convert a cell coordinate "on the circle" to a position "on the chunk"
    /// Return an Err if this is not on the chunk
    pub fn absolute_cell_idx_to_in_chunk_cell_idx(
        &self,
        cell_idx: IjkVector,
    ) -> Result<JkVector, String> {
        if cell_idx.i != self.get_layer_num() {
            return Err(format!(
                "Cell index i {:?} is not in chunk {:?}",
                cell_idx,
                self.get_chunk_idx()
            ));
        }
        let start_radial_line = self.get_start_radial_line();
        let end_radial_line = self.get_end_radial_line();
        let start_concentric_circle = self.get_start_concentric_circle_layer_relative();
        let end_concentric_circle = self.get_end_concentric_circle_layer_relative();
        if cell_idx.j < start_concentric_circle || cell_idx.j >= end_concentric_circle {
            return Err(format!(
                "Cell index j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                cell_idx,
                self.get_chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            ));
        }
        if cell_idx.k < start_radial_line || cell_idx.k >= end_radial_line {
            return Err(format!(
                "Cell index k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                cell_idx,
                self.get_chunk_idx(),
                start_radial_line,
                end_radial_line,
            ));
        }
        Ok(JkVector {
            j: cell_idx.j - start_concentric_circle,
            k: cell_idx.k - start_radial_line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
    use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};
    use crate::physics::util::vectors::RelXyPoint;

    /// Iterate around the circle in every direction, targetting each cells midpoint, and make sure
    /// the cell index is correct returned by rel_pos_to_cell_idx
    #[test]
    fn test_rel_pos_to_cell_idx() {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(8)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();

        // Test the core
        let i = 0;
        let j = 0;
        let core_chunks = coordinate_dir.get_core_chunks();
                let num_radial_lines = core_chunks.get_width()*core_chunks.get(JkVector::ZERO).get_num_radial_lines();
        for k in 0..num_radial_lines {
            // This radius and theta should define the midpoint of each cell
            let radius = coordinate_dir.get_cell_width() / 2.0;
            let theta = -2.0 * PI / num_radial_lines as f32
                * (k as f32 + 0.5);
            let xycoord = RelXyPoint(Vec2 {
                x: radius * theta.cos(),
                y: radius * theta.sin(),
            });
            let cell_idx = coordinate_dir.rel_pos_to_cell_idx(xycoord).unwrap();
            let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(cell_idx);
            let chunk = coordinate_dir.get_chunk_at_idx(chunk_idx.0);
            assert_eq!(
                chunk.rel_pos_to_cell_idx(xycoord).unwrap(),
                IjkVector { i, j, k },
                "k: {}, radius: {}, theta: {}, xycoord: {:?}",
                k,
                radius,
                theta,
                xycoord
            );
        }

        // Test the rest
        for i in 1..coordinate_dir.get_num_layers() {
            let num_concentric_circles = coordinate_dir.get_layer_num_concentric_circles(i);
            let num_radial_lines = coordinate_dir.get_layer_num_radial_lines(i);
            for j in 0..num_concentric_circles {
                for k in 0..num_radial_lines {
                    // This radius and theta should define the midpoint of each cell
                    let radius = coordinate_dir.get_layer_start_radius(i)
                        + (coordinate_dir.get_layer_end_radius(i)
                            - coordinate_dir.get_layer_start_radius(i))
                            / num_concentric_circles as f32
                            * (j as f32 + 0.5);
                    let theta = -2.0 * PI / num_radial_lines as f32 * (k as f32 + 0.5);
                    let xycoord = RelXyPoint(Vec2 {
                        x: radius * theta.cos(),
                        y: radius * theta.sin(),
                    });
                    let cell_idx = coordinate_dir.rel_pos_to_cell_idx(xycoord).unwrap();
                    let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(cell_idx);
                    let chunk = coordinate_dir.get_chunk_at_idx(chunk_idx.0);
                    assert_eq!(
                        chunk.rel_pos_to_cell_idx(xycoord).unwrap(),
                        IjkVector { i, j, k }
                    );
                }
            }
        }
    }

    #[test]
    fn test_cell_idx_to_chunk_idx() {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(8)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();

        // Test the core
        let i = 0;
        let j = 0;
        let core_chunks = coordinate_dir.get_core_chunks();
        let num_radial_lines = core_chunks.get_width()*core_chunks.get(JkVector::ZERO).get_num_radial_lines();
        for k in 0..num_radial_lines {
            // This radius and theta should define the midpoint of each cell
            let coord = IjkVector { i, j, k: k % core_chunks.get(JkVector::ZERO).get_num_radial_lines() };
            let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(coord);
            let chunk = coordinate_dir.get_chunk_at_idx(chunk_idx.0);
            assert_eq!(
                chunk.absolute_cell_idx_to_in_chunk_cell_idx(coord),
                Ok(coord.to_jk_vector())
            );
        }

        // Test the rest
        for i in 1..coordinate_dir.get_num_layers() {
            let num_concentric_chunks = coordinate_dir.get_layer_num_concentric_chunks(i);
            let num_radial_chunks = coordinate_dir.get_layer_num_radial_chunks(i);
            let mut total_concentric_circles = 0;
            for cj in 0..num_concentric_chunks {
                let mut total_radial_lines = 0;
                let chunk_layer_num_concentric_circles = coordinate_dir
                    .get_chunk_num_concentric_circles(ChunkIjkVector { i, j: cj, k: 0 });
                for ck in 0..num_radial_chunks {
                    let chunk_num_radial_lines = coordinate_dir
                        .get_chunk_num_radial_lines(ChunkIjkVector { i, j: cj, k: ck });
                    for j in total_concentric_circles
                        ..total_concentric_circles + chunk_layer_num_concentric_circles
                    {
                        for k in total_radial_lines..total_radial_lines + chunk_num_radial_lines {
                            let absolute_coord = IjkVector { i, j, k };
                            let in_chunk_coord = JkVector {
                                j: j - total_concentric_circles,
                                k: k - total_radial_lines,
                            };
                            let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(absolute_coord);
                            // assert_eq!(chunk_idx, ChunkIjkVector { i, j: cj, k: ck });
                            let chunk = coordinate_dir.get_chunk_at_idx(chunk_idx.0);
                            assert_eq!(
                                chunk.absolute_cell_idx_to_in_chunk_cell_idx(absolute_coord),
                                Ok(in_chunk_coord)
                            );
                        }
                    }
                    total_radial_lines += chunk_num_radial_lines;
                }
                total_concentric_circles += chunk_layer_num_concentric_circles;
            }
        }
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

    mod full_layer {
        use super::*;

        const FIRST_LAYER: ChunkCoords = ChunkCoords {
            width: 1.0,
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
            let vertices = FIRST_LAYER.get_positions(VertexMode::Lines);
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
            let uvs = FIRST_LAYER.get_uvs(VertexMode::Lines);
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
            let indices = FIRST_LAYER.get_indices(VertexMode::Lines);
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

        const FIRST_LAYER_PARTIAL: ChunkCoords = ChunkCoords {
            width: 1.0,
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
            let vertices = FIRST_LAYER_PARTIAL.get_positions(VertexMode::Lines);
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
            let uvs = FIRST_LAYER_PARTIAL.get_uvs(VertexMode::Lines);
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
