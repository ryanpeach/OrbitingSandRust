#![expect(missing_docs)]
#![expect(clippy::missing_docs_in_private_items)]
use crate::physics::fallingsand::util::functions::interpolate_points;

use crate::common::util::mesh::OwnedMeshData;
use crate::common::util::mesh::Vertex;
use crate::common::util::vectors::ModelCoord;
use crate::physics::fallingsand::util::vectors::{ChunkIjkVector, IjkVector, JkVector};
use crate::physics::orbits::components::Length;
use anyhow::{bail, Result};
use bevy::color::Color;
use bevy::math::{Rect, Vec2};
use conv::{ConvAsUtil, ValueFrom};
use std::f64::consts::PI;

/// The settings for generating the vertexes
#[derive(Debug, Clone, Copy)]
pub struct VertexSettings {
    pub lod: u32,
    pub mode: VertexMode,
}

impl Default for VertexSettings {
    fn default() -> Self {
        Self {
            lod: 1,
            mode: VertexMode::Lines,
        }
    }
}

impl VertexSettings {
    /// Create a grid of vertexes with a given level of detail
    /// The level of detail will be the number of vertexes skipped in the grid
    /// The level of detail must be a power of 2
    /// This is useful for drawing the grid zoomed out, or for drawing the player grid
    #[must_use]
    pub fn grid(lod: u32) -> VertexSettings {
        debug_assert!(lod > 0);
        debug_assert_eq!(lod & (lod - 1), 0, "lod must be a power of 2");
        VertexSettings {
            lod,
            mode: VertexMode::Grid,
        }
    }
}

/// The optimal way of drawing the vertexes is just to draw the radial lines.
/// Because the texture will map along the column perfectly.
/// However, if you want to see the whole grid you can use the grid vertexes.
#[derive(Default, Debug, Clone, Copy)]
pub enum VertexMode {
    #[default]
    Lines,
    Grid,
}

/// This is a chunk that represents a "full" layer.
/// It doesn't split itself in either the tangential or radial directions.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkCoords {
    cell_width: Length,
    chunk_idx: ChunkIjkVector,
    start_concentric_circle_layer_relative: u32,
    start_concentric_circle_absolute: u32,
    start_radial_line: u32,
    end_radial_line: u32,
    layer_num_radial_lines: u32,
    num_concentric_circles: u32,
}

pub struct PartialLayerChunkCoordsBuilder {
    cell_width: Length,
    chunk_idx: ChunkIjkVector,
    start_concentric_circle_layer_relative: u32,
    start_concentric_circle_absolute: u32,
    start_radial_line: u32,
    end_radial_line: u32,
    layer_num_radial_lines: u32,
    num_concentric_circles: u32,
}

impl Default for PartialLayerChunkCoordsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialLayerChunkCoordsBuilder {
    /// Defaults to first layer defaults
    #[must_use]
    pub fn new() -> PartialLayerChunkCoordsBuilder {
        PartialLayerChunkCoordsBuilder {
            cell_width: Length(1.0),
            chunk_idx: ChunkIjkVector::ZERO,
            start_concentric_circle_layer_relative: 0,
            start_concentric_circle_absolute: 0,
            start_radial_line: 0,
            end_radial_line: 0,
            layer_num_radial_lines: 0,
            num_concentric_circles: 0,
        }
    }

    /// Set the cell radius
    #[must_use]
    pub fn cell_width(mut self, cell_width: Length) -> PartialLayerChunkCoordsBuilder {
        debug_assert!(cell_width.0 > 0.0);
        self.cell_width = cell_width;
        self
    }

    /// Set the index of the first concentric circle starting from the beginning of the layer
    #[must_use]
    pub fn start_concentric_circle_layer_relative(
        mut self,
        start_concentric_circle_layer_relative: u32,
    ) -> PartialLayerChunkCoordsBuilder {
        self.start_concentric_circle_layer_relative = start_concentric_circle_layer_relative;
        self
    }

    /// Set the index of the first concentric circle starting from the center of the circle
    #[must_use]
    pub fn start_concentric_circle_absolute(
        mut self,
        start_concentric_circle_absolute: u32,
    ) -> PartialLayerChunkCoordsBuilder {
        self.start_concentric_circle_absolute = start_concentric_circle_absolute;
        self
    }

    /// Set the index of the first radial line in the chunk
    #[must_use]
    pub fn start_radial_line(mut self, start_radial_line: u32) -> PartialLayerChunkCoordsBuilder {
        self.start_radial_line = start_radial_line;
        self
    }

    /// Set the index of the last radial line in the chunk
    #[must_use]
    pub fn end_radial_line(mut self, end_radial_line: u32) -> PartialLayerChunkCoordsBuilder {
        self.end_radial_line = end_radial_line;
        self
    }

    #[must_use]
    pub fn chunk_idx(mut self, chunk_idx: ChunkIjkVector) -> PartialLayerChunkCoordsBuilder {
        self.chunk_idx = chunk_idx;
        self
    }

    #[must_use]
    pub fn layer_num_radial_lines(
        mut self,
        layer_num_radial_lines: u32,
    ) -> PartialLayerChunkCoordsBuilder {
        debug_assert_ne!(layer_num_radial_lines, 0);
        self.layer_num_radial_lines = layer_num_radial_lines;
        self
    }

    #[must_use]
    pub fn num_concentric_circles(
        mut self,
        num_concentric_circles: u32,
    ) -> PartialLayerChunkCoordsBuilder {
        debug_assert_ne!(num_concentric_circles, 0);
        self.num_concentric_circles = num_concentric_circles;
        self
    }

    #[must_use]
    pub fn build(self) -> ChunkCoords {
        debug_assert!(self.end_radial_line > self.start_radial_line);
        debug_assert!(self.end_radial_line <= self.layer_num_radial_lines);
        debug_assert_ne!(self.num_concentric_circles, 0);
        debug_assert_ne!(self.layer_num_radial_lines, 0);
        debug_assert_ne!(self.end_radial_line, 0);
        ChunkCoords {
            cell_width: self.cell_width,
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
    /// Get the vertex positions for the chunk
    #[must_use]
    pub fn positions(&self, settings: VertexSettings) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric_circle = self.start_concentric_circle_layer_relative;

        let starting_r = f64::from(self.start_radius());
        let ending_r = f64::from(self.end_radius());
        let circle_separation_distance =
            (ending_r - starting_r) / f64::from(self.num_concentric_circles());
        let theta = (-2.0 * PI) / f64::from(self.layer_num_radial_lines);

        // Create the concentric range with the appropriate level of detail and test it has the right bounds
        let start_concentric = self.start_concentric_circle_layer_relative;
        let mut concentric_range: Vec<u32> = match settings.mode {
            VertexMode::Lines => vec![
                start_concentric_circle,
                self.num_concentric_circles() + start_concentric_circle,
            ],
            VertexMode::Grid => (start_concentric
                ..=(self.num_concentric_circles() + start_concentric))
                .step_by(settings.lod as usize)
                .collect(),
        };
        debug_assert_eq!(concentric_range[0], start_concentric);
        if concentric_range[concentric_range.len() - 1]
            != self.num_concentric_circles() + start_concentric
        {
            concentric_range.push(self.num_concentric_circles() + start_concentric);
        }
        debug_assert_eq!(
            concentric_range[concentric_range.len() - 1],
            self.num_concentric_circles() + start_concentric
        );

        // Create the radial range with the appropriate level of detail and test it has the right bounds
        let mut radial_range: Vec<u32> = (self.start_radial_line..=(self.end_radial_line))
            .step_by(settings.lod as usize)
            .collect();
        debug_assert_eq!(radial_range[0], self.start_radial_line);
        if radial_range[radial_range.len() - 1] != self.end_radial_line {
            radial_range.push(self.end_radial_line);
        }
        debug_assert_eq!(radial_range[radial_range.len() - 1], self.end_radial_line);

        // Run our double loop
        for j in concentric_range {
            let diff = f64::from(j - self.start_concentric_circle_layer_relative)
                * circle_separation_distance;

            for k in &radial_range {
                if j == 0 && k % 2 == 1 {
                    let angle_next = f64::from(k + 1) * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().expect("vertexes is non empty");
                    let v_next = Vec2::new(
                        (angle_next.cos() * radius)
                            .approx()
                            .expect("This is not a strange conversion"),
                        (angle_next.sin() * radius)
                            .approx()
                            .expect("This is not a strange conversion"),
                    );
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else {
                    let angle_point = f64::from(*k) * theta;
                    let radius = starting_r + diff;
                    let new_coord = Vec2::new(
                        (angle_point.cos() * radius)
                            .approx()
                            .expect("This is not a strange conversion"),
                        (angle_point.sin() * radius)
                            .approx()
                            .expect("This is not a strange conversion"),
                    );
                    vertexes.push(new_coord);
                }
            }
        }

        vertexes
    }

    /// Similar to `get_circle_vertexes`, but the j index just iterates on the 0th and last element
    #[must_use]
    pub fn outline(&self) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let start_concentric_circle = self.start_concentric_circle_layer_relative;
        let start_radial_line = self.start_radial_line;

        let starting_r = f64::from(self.start_radius());
        let ending_r = f64::from(self.end_radius());
        let circle_separation_distance =
            (ending_r - starting_r) / f64::from(self.num_concentric_circles());
        let theta = (-2.0 * PI) / f64::from(self.layer_num_radial_lines);

        for j in [
            start_concentric_circle,
            self.num_concentric_circles() + start_concentric_circle,
        ] {
            let diff = f64::from(j - start_concentric_circle) * circle_separation_distance;

            // Reverse if we are on the last element because we are going around the circle
            // This box method was the only way to make Range == Rev<Range> in type, very annoying.
            let iter: Box<dyn Iterator<Item = _>> = if j == start_concentric_circle {
                Box::new(start_radial_line..=self.end_radial_line)
            } else {
                Box::new((start_radial_line..=self.end_radial_line).rev())
            };

            for k in iter {
                if j == 0 && k % 2 == 1 {
                    let angle_next = f64::from(k + 1) * theta;
                    let radius = starting_r + diff;
                    let v_last = vertexes.last().expect("vertexes is not empty");
                    let v_next = Vec2::new(
                        (angle_next.cos() * radius)
                            .approx()
                            .expect("This is not a strange conversion."),
                        (angle_next.sin() * radius)
                            .approx()
                            .expect("This is not a strange conversion."),
                    );
                    vertexes.push(interpolate_points(v_last, &v_next));
                } else {
                    let angle_point = f64::from(k) * theta;
                    let radius = starting_r + diff;
                    let new_coord = Vec2::new(
                        (angle_point.cos() * radius)
                            .approx()
                            .expect("This is not a strange conversion."),
                        (angle_point.sin() * radius)
                            .approx()
                            .expect("This is not a strange conversion."),
                    );
                    vertexes.push(new_coord);
                }
            }
        }

        vertexes
    }

    /// Gets the min and max positions in raw x, y of the chunk
    pub fn bounding_box(&self) -> Rect {
        let outline = self.outline();
        let all_x = outline.iter().map(|v| v.x);
        let all_y = outline.iter().map(|v| v.y);
        let min_x = all_x.clone().fold(f32::INFINITY, f32::min);
        let max_x = all_x.fold(f32::NEG_INFINITY, f32::max);
        let min_y = all_y.clone().fold(f32::INFINITY, f32::min);
        let max_y = all_y.fold(f32::NEG_INFINITY, f32::max);
        Rect::new(min_x, min_y, max_x, max_y)
    }

    /// Gets the UV coordinates of the vertexes of the chunk
    /// This is a more traditional square grid
    /// If you set skip to 1, you will get the full resolution
    /// If you set skip to 2, you will get half the resolution
    /// ...
    #[must_use]
    pub fn uvs(&self, settings: VertexSettings) -> Vec<Vec2> {
        let mut vertexes: Vec<Vec2> = Vec::new();

        let mut concentric_range: Vec<u32> = match settings.mode {
            VertexMode::Lines => vec![0, self.num_concentric_circles()],
            VertexMode::Grid => (0..=self.num_concentric_circles())
                .step_by(settings.lod as usize)
                .collect::<Vec<_>>(),
        };
        debug_assert_eq!(concentric_range[0], 0);
        if concentric_range[concentric_range.len() - 1] != self.num_concentric_circles() {
            concentric_range.push(self.num_concentric_circles());
        }
        debug_assert_eq!(
            concentric_range[concentric_range.len() - 1],
            self.num_concentric_circles()
        );

        for j in concentric_range {
            for k in (0..=self.num_radial_lines()).step_by(settings.lod as usize) {
                let new_vec = Vec2::new(
                    (f64::from(k) / f64::from(self.num_radial_lines()))
                        .approx()
                        .expect("This is not a strange conversion."),
                    (f64::from(j) / f64::from(self.num_concentric_circles()))
                        .approx()
                        .expect("This is not a strange conversion."),
                );
                vertexes.push(new_vec);
            }
        }

        vertexes
    }

    /// Creates the indices for the vertexes
    #[must_use]
    pub fn indices(&self, settings: VertexSettings) -> Vec<u32> {
        let mut j_count: u32 = match settings.mode {
            VertexMode::Lines => 2,
            VertexMode::Grid => self.num_concentric_circles() / settings.lod + 1,
        };
        j_count = j_count.max(2);
        let k_iter: Vec<u32> = (0..=self.num_radial_lines())
            .step_by(settings.lod as usize)
            .collect();
        let k_count = u32::value_from(k_iter.len())
            .expect("This equals self.num_radial_lines(), which is u32");
        let mut indices = Vec::with_capacity(j_count as usize * k_count as usize * 6);
        for j in 0..j_count - 1 {
            for k in 0..k_count - 1 {
                // Compute the four corners of our current grid cell
                let v0 = j * k_count + k; // Top-left
                let v1 = v0 + 1; // Top-right
                let v2 = v0 + k_count + 1; // Bottom-right
                let v3 = v0 + k_count; // Bottom-left

                // First triangle (top-left, bottom-left, top-right)
                indices.push(v0);
                indices.push(v3);
                indices.push(v1);

                // Second triangle (top-right, bottom-left, bottom-right)
                indices.push(v1);
                indices.push(v3);
                indices.push(v2);
            }
        }

        indices
    }
}

impl ChunkCoords {
    /// Get the total number of cells in the chunk
    #[must_use]
    pub fn total_size(&self) -> u32 {
        self.num_radial_lines() * self.num_concentric_circles()
    }
    /// Get the width of a cell (which is a square in this case)
    #[must_use]
    pub fn cell_width(&self) -> Length {
        self.cell_width
    }
    /// Get the radius of the smallest concentric circle
    #[must_use]
    pub fn start_radius(&self) -> f32 {
        (f64::from(self.start_concentric_circle_absolute) * f64::from(self.cell_width.0))
            .approx()
            .expect("This is not a strange conversion.")
    }
    /// Get the radius of the largest concentric circle
    #[must_use]
    pub fn end_radius(&self) -> f32 {
        (f64::from(self.start_radius())
            + f64::from(self.cell_width.0) * f64::from(self.num_concentric_circles))
        .approx()
        .expect("This is not a strange conversion.")
    }
    /// Get the number of radial lines in the chunk
    /// These go around the circle counter clockwise
    #[must_use]
    pub fn num_radial_lines(&self) -> u32 {
        self.end_radial_line - self.start_radial_line
    }
    /// Get the number of concentric circles in the chunk
    /// These go from the center of the circle to the edge
    #[must_use]
    pub fn num_concentric_circles(&self) -> u32 {
        self.num_concentric_circles
    }
    #[must_use]
    pub fn end_theta(&self) -> f32 {
        let diff = (2.0 * PI) / f64::from(self.layer_num_radial_lines);
        (f64::from(self.end_radial_line) * diff)
            .approx()
            .expect("This is not a strange conversion.")
    }
    #[must_use]
    pub fn start_theta(&self) -> f32 {
        let diff = (2.0 * PI) / f64::from(self.layer_num_radial_lines);
        (f64::from(self.start_radial_line) * diff)
            .approx()
            .expect("This is not a strange conversion.")
    }
    /// Get the index of the first concentric circle starting from the beginning of the layer
    #[must_use]
    pub fn start_concentric_circle_layer_relative(&self) -> u32 {
        self.start_concentric_circle_layer_relative
    }
    /// Get the index of the first concentric circle starting from the center of the circle
    #[must_use]
    pub fn start_concentric_circle_absolute(&self) -> u32 {
        self.start_concentric_circle_absolute
    }
    /// Get the index of the last concentric circle starting from the center of the circle
    /// This will be one greater than the last cell index, because it has to enclose the cell
    #[must_use]
    pub fn end_concentric_circle_absolute(&self) -> u32 {
        self.start_concentric_circle_absolute + self.num_concentric_circles
    }
    /// Get the index of the last concentric circle starting from the beginning of the layer
    /// This will be one greater than the last cell index, because it has to enclose the cell
    #[must_use]
    pub fn end_concentric_circle_layer_relative(&self) -> u32 {
        self.start_concentric_circle_layer_relative + self.num_concentric_circles
    }
    /// Get the index of the last radial line in the chunk
    /// This will be one greater than the last cell index, because it has to enclose the cell
    #[must_use]
    pub fn end_radial_line(&self) -> u32 {
        self.end_radial_line
    }
    /// Get the index of the first radial line in the chunk
    #[must_use]
    pub fn start_radial_line(&self) -> u32 {
        self.start_radial_line
    }
    /// Get the layer number this chunk is a part of
    #[must_use]
    pub fn layer_num(&self) -> u32 {
        self.chunk_idx.i
    }
    /// Get the chunk index
    #[must_use]
    pub fn chunk_idx(&self) -> ChunkIjkVector {
        self.chunk_idx
    }

    /* Positions in the chunk */
    /// Checks to see if an absolute position around the circle is in the chunk
    #[must_use]
    pub fn contains(&self, idx: IjkVector) -> bool {
        idx.i == self.layer_num()
            && idx.j >= self.start_radial_line()
            && idx.j < self.end_radial_line()
            && idx.k >= self.start_concentric_circle_absolute()
            && idx.k < self.end_concentric_circle_absolute()
    }
    /// Converts a coordinate from anywhere on the circle, assuming it is in the chunk
    /// to a coordinate inside the grid of this chunk
    #[must_use]
    pub fn internal_coord_from_external_coord(&self, external_coord: IjkVector) -> JkVector {
        debug_assert!(self.contains(external_coord));
        JkVector {
            j: external_coord.j - self.start_radial_line(),
            k: external_coord.k - self.start_concentric_circle_absolute(),
        }
    }
    /// Converts a coordinate from inside this chunk to a coordinate on the circle
    #[must_use]
    pub fn external_coord_from_internal_coord(&self, internal_coord: JkVector) -> IjkVector {
        debug_assert!(internal_coord.j < self.num_radial_lines());
        debug_assert!(internal_coord.k < self.num_concentric_circles());
        IjkVector {
            i: self.layer_num(),
            j: internal_coord.j + self.start_radial_line(),
            k: internal_coord.k + self.start_concentric_circle_absolute(),
        }
    }

    /* Convienience Functions */
    /// Get all the vertexes for the chunk
    #[must_use]
    pub fn vertices(&self, settings: VertexSettings) -> Vec<Vertex> {
        let positions = self.positions(settings);
        let uvs = self.uvs(settings);
        let vertexes: Vec<Vertex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(p, uv)| Vertex {
                position: Vec2::new(p.x, p.y) * self.cell_width().0,
                uv: Vec2::new(uv.x, uv.y),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            })
            .collect();
        vertexes
    }
    /// Get the outline mesh for the chunk
    #[must_use]
    pub fn chunk_outline(&self) -> OwnedMeshData {
        let positions = self.outline();
        let mut vertices = Vec::with_capacity(positions.len());
        for pos in positions {
            vertices.push(Vertex {
                position: pos * self.cell_width().0,
                uv: Vec2::new(0.0, 0.0),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            });
        }
        let mut indices = Vec::new();
        for i in 0..vertices.len() {
            indices.push(
                u32::value_from(i).expect("There are not more than u32 vertices in a single chunk"),
            );
        }
        OwnedMeshData::new(vertices, indices)
    }
    /// Get the mesh data for the chunk as you would normally draw it
    #[must_use]
    pub fn chunk_meshdata(&self, settings: VertexSettings) -> OwnedMeshData {
        let indices = self.indices(settings);
        let vertices: Vec<Vertex> = self.vertices(settings);
        OwnedMeshData::new(vertices, indices)
    }

    /// Get the wireframe mesh data for the chunk
    #[must_use]
    pub fn chunk_triangle_wireframe(&self, settings: VertexSettings) -> OwnedMeshData {
        let indices = self.indices(settings);
        let vertices: Vec<Vertex> = self.vertices(settings);
        let mut new_indices = Vec::new();
        for i in (0..indices.len()).step_by(3) {
            let i1 = indices[i];
            let i2 = indices[i + 1];
            let i3 = indices[i + 2];

            new_indices.push(i1);
            new_indices.push(i2);
            new_indices.push(i3);
        }
        OwnedMeshData::new(vertices, new_indices)
    }

    /// Converts a position relative to the origin of the circle to a cell index
    /// Returns an Err if the position is not on the circle
    pub fn rel_pos_to_cell_idx(&self, xy_coord: ModelCoord) -> Result<IjkVector> {
        let norm_vertex_coord = (xy_coord.0.x * xy_coord.0.x + xy_coord.0.y * xy_coord.0.y).sqrt();
        let start_concentric_circle = self.start_concentric_circle_layer_relative();
        let end_concentric_circle = self.end_concentric_circle_layer_relative();
        let starting_r = self.start_radius();
        let ending_r = self.end_radius();
        let num_concentric_circles = self.num_concentric_circles();
        let num_radial_lines = self.num_radial_lines();
        let start_radial_line = self.start_radial_line();
        let end_radial_line = self.end_radial_line();
        let start_theta = self.start_theta();
        let end_theta = self.end_theta();

        // Get the concentric circle we are on
        let circle_separation_distance =
            f64::from(ending_r - starting_r) / f64::from(num_concentric_circles);

        // Calculate 'j' directly without the while loop
        let j_rel: u32 = (f64::from(norm_vertex_coord - starting_r) / circle_separation_distance)
            .floor()
            .approx()
            .expect("j_rel wont be larger than the number of concentric circles in a layer");
        let j = j_rel.min(end_concentric_circle - 1) + start_concentric_circle;

        // Get the radial line to the left of the vertex
        let angle =
            (f64::from(xy_coord.0.y).atan2(f64::from(xy_coord.0.x)) + -2.0 * PI) % (2.0 * PI);
        let theta = -(f64::from(end_theta) - f64::from(start_theta)) / f64::from(num_radial_lines);

        // Calculate 'k' directly without the while loop
        let k_rel: u32 = (angle / theta)
            .floor()
            .approx()
            .expect("k_rel wont be larger than the number of radial lines in a layer");
        let k = k_rel.min(end_radial_line - 1);

        // Check to see if the vertex is in the chunk
        if j < start_concentric_circle && j >= end_concentric_circle {
            bail!(
                "Vertex j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                xy_coord,
                self.chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            );
        }
        if k < start_radial_line && k >= end_radial_line {
            bail!(
                "Vertex k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                xy_coord,
                self.chunk_idx(),
                start_radial_line,
                end_radial_line,
            );
        }
        Ok(IjkVector {
            i: self.layer_num(),
            j,
            k,
        })
    }

    /// Convert a cell coordinate "on the circle" to a position "on the chunk"
    /// Return an Err if this is not on the chunk
    pub fn absolute_cell_idx_to_in_chunk_cell_idx(&self, cell_idx: IjkVector) -> Result<JkVector> {
        if cell_idx.i != self.layer_num() {
            bail!(
                "Cell index i {:?} is not in chunk {:?}",
                cell_idx,
                self.chunk_idx()
            );
        }
        let start_radial_line = self.start_radial_line();
        let end_radial_line = self.end_radial_line();
        let start_concentric_circle = self.start_concentric_circle_layer_relative();
        let end_concentric_circle = self.end_concentric_circle_layer_relative();
        if cell_idx.j < start_concentric_circle || cell_idx.j >= end_concentric_circle {
            bail!(
                "Cell index j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                cell_idx,
                self.chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            );
        }
        if cell_idx.k < start_radial_line || cell_idx.k >= end_radial_line {
            bail!(
                "Cell index k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                cell_idx,
                self.chunk_idx(),
                start_radial_line,
                end_radial_line,
            );
        }
        Ok(JkVector {
            j: cell_idx.j - start_concentric_circle,
            k: cell_idx.k - start_radial_line,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::unwrap_used,
        clippy::panic,
        clippy::float_cmp
    )]
    use super::*;

    use crate::physics::fallingsand::mesh::coordinate_dir::Builder;
    use crate::physics::fallingsand::util::vectors::{IjkVector, JkVector};
    use crate::physics::util::vectors::RelXyPoint;

    /// Iterate around the circle in every direction, targetting each cells midpoint, and make sure
    /// the cell index is correct returned by `rel_pos_to_cell_idx`
    #[test]
    fn test_rel_pos_to_cell_idx() {
        let coordinate_dir = Builder::new()
            .cell_width(Length(1.0))
            .num_layers(8)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();

        // Test the core
        let i = 0;
        let j = 0;
        let core_chunks = coordinate_dir.core_chunks();
        let num_radial_lines =
            core_chunks.width() * core_chunks.get(JkVector::ZERO).num_radial_lines();
        for k in 0..num_radial_lines {
            // This radius and theta should define the midpoint of each cell
            let radius = f64::from(coordinate_dir.cell_width().0) / 2.0;
            let theta = -2.0 * PI / f64::from(num_radial_lines) * (f64::from(k) + 0.5);
            let xycoord = ModelCoord(Vec2 {
                x: (radius * theta.cos()) as f32,
                y: (radius * theta.sin()) as f32,
            });
            let cell_idx = coordinate_dir.rel_pos_to_cell_idx(xycoord).unwrap();
            let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(cell_idx);
            let chunk = coordinate_dir.chunk_at_idx(chunk_idx.0);
            assert_eq!(
                chunk.rel_pos_to_cell_idx(xycoord).unwrap(),
                IjkVector { i, j, k },
                "k: {k}, radius: {radius}, theta: {theta}, xycoord: {xycoord:?}"
            );
        }

        // Test the rest
        for i in 1..coordinate_dir.num_layers() {
            let num_concentric_circles = coordinate_dir.layer_num_concentric_circles(i);
            let num_radial_lines = coordinate_dir.layer_num_radial_lines(i);
            for j in 0..num_concentric_circles {
                for k in 0..num_radial_lines {
                    // This radius and theta should define the midpoint of each cell
                    let radius: f64 = f64::from(coordinate_dir.layer_start_radius(i))
                        + (f64::from(coordinate_dir.layer_end_radius(i))
                            - f64::from(coordinate_dir.layer_start_radius(i)))
                            / f64::from(num_concentric_circles)
                            * (f64::from(j) + 0.5);
                    let theta = -2.0 * PI / f64::from(num_radial_lines) * (f64::from(k) + 0.5);
                    let xycoord = ModelCoord(Vec2 {
                        x: (radius * theta.cos()).approx().unwrap(),
                        y: (radius * theta.sin()).approx().unwrap(),
                    });
                    let cell_idx = coordinate_dir.rel_pos_to_cell_idx(xycoord).unwrap();
                    let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(cell_idx);
                    let chunk = coordinate_dir.chunk_at_idx(chunk_idx.0);
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
        let coordinate_dir = Builder::new()
            .cell_width(Length(1.0))
            .num_layers(8)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();

        // Test the core
        let i = 0;
        let j = 0;
        let core_chunks = coordinate_dir.core_chunks();
        let num_radial_lines =
            core_chunks.width() * core_chunks.get(JkVector::ZERO).num_radial_lines();
        for k in 0..num_radial_lines {
            // This radius and theta should define the midpoint of each cell
            let coord = IjkVector {
                i,
                j,
                k: k % core_chunks.get(JkVector::ZERO).num_radial_lines(),
            };
            let chunk_idx = coordinate_dir.cell_idx_to_chunk_idx(coord);
            let chunk = coordinate_dir.chunk_at_idx(chunk_idx.0);
            assert_eq!(
                chunk.absolute_cell_idx_to_in_chunk_cell_idx(coord).unwrap(),
                coord.to_jk_vector()
            );
        }

        // Test the rest
        for i in 1..coordinate_dir.num_layers() {
            let num_concentric_chunks = coordinate_dir.layer_num_concentric_chunks(i);
            let num_tangential_chunkss = coordinate_dir.layer_num_tangential_chunkss(i);
            let mut total_concentric_circles = 0;
            for cj in 0..num_concentric_chunks {
                let mut total_radial_lines = 0;
                let chunk_layer_num_concentric_circles =
                    coordinate_dir.chunk_num_concentric_circles(ChunkIjkVector { i, j: cj, k: 0 });
                for ck in 0..num_tangential_chunkss {
                    let chunk_num_radial_lines =
                        coordinate_dir.chunk_num_radial_lines(ChunkIjkVector { i, j: cj, k: ck });
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
                            let chunk = coordinate_dir.chunk_at_idx(chunk_idx.0);
                            assert_eq!(
                                chunk
                                    .absolute_cell_idx_to_in_chunk_cell_idx(absolute_coord)
                                    .unwrap(),
                                in_chunk_coord
                            );
                        }
                    }
                    total_radial_lines += chunk_num_radial_lines;
                }
                total_concentric_circles += chunk_layer_num_concentric_circles;
            }
        }
    }

    pub fn vec2_approx_eq(a: Vec2, b: Vec2, epsilon: f32) -> bool {
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

        pub const FIRST_LAYER: ChunkCoords = ChunkCoords {
            cell_width: Length(1.0),
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
            let vertices = FIRST_LAYER.positions(VertexSettings::default());
            assert_eq!(vertices.len(), 13 * 2);

            // The inner circle
            // every other vertex is actually an interpolation of the previous layer's num_radial_lines
            let radius: f64 = 1.0;
            let num_radial_lines = 12;
            assert_approx_eq_v2!(vertices[0], Vec2::new(radius as f32, 0.0));
            assert_approx_eq_v2!(vertices[1], interpolate_points(&vertices[0], &vertices[2]));
            assert_approx_eq_v2!(
                vertices[2],
                Vec2::new(
                    (radius * (2.0 * PI * -2.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -2.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(vertices[3], interpolate_points(&vertices[2], &vertices[4]));
            assert_approx_eq_v2!(
                vertices[4],
                Vec2::new(
                    (radius * (2.0 * PI * -4.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -4.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(vertices[5], interpolate_points(&vertices[4], &vertices[6]));
            assert_approx_eq_v2!(
                vertices[6],
                Vec2::new(
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(vertices[7], interpolate_points(&vertices[6], &vertices[8]));
            assert_approx_eq_v2!(
                vertices[8],
                Vec2::new(
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(vertices[9], interpolate_points(&vertices[8], &vertices[10]));
            assert_approx_eq_v2!(
                vertices[10],
                Vec2::new(
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[11],
                interpolate_points(&vertices[10], &vertices[12])
            );
            assert_approx_eq_v2!(
                vertices[12],
                Vec2::new(
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );

            // The outer circle
            let radius: f64 = 3.0;
            let num_radial_lines = 12;
            assert_approx_eq_v2!(vertices[13], Vec2::new(radius as f32, 0.0));
            assert_approx_eq_v2!(
                vertices[14],
                Vec2::new(
                    (radius * (2.0 * PI * -1.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -1.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[15],
                Vec2::new(
                    (radius * (2.0 * PI * -2.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -2.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[16],
                Vec2::new(
                    (radius * (2.0 * PI * -3.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -3.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[17],
                Vec2::new(
                    (radius * (2.0 * PI * -4.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -4.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[18],
                Vec2::new(
                    (radius * (2.0 * PI * -5.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -5.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[19],
                Vec2::new(
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[20],
                Vec2::new(
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[21],
                Vec2::new(
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[22],
                Vec2::new(
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[23],
                Vec2::new(
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[24],
                Vec2::new(
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[25],
                Vec2::new(
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
        }

        #[test]
        fn test_first_layer_uv() {
            let uvs = FIRST_LAYER.uvs(VertexSettings::default());
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
            let indices = FIRST_LAYER.indices(VertexSettings::default());
            assert_eq!(indices.len(), 12 * 6);

            // The first concentric circle
            let mut j = 0;
            for i in 0..12u32 {
                assert_eq!(indices[j], i, "i: {i}");
                assert_eq!(indices[j + 1], i + 13u32, "i: {i}");
                assert_eq!(indices[j + 2], i + 1u32, "i: {i}");
                assert_eq!(indices[j + 3], i + 1u32, "i: {i}");
                assert_eq!(indices[j + 4], i + 13u32, "i: {i}");
                assert_eq!(indices[j + 5], i + 14u32, "i: {i}");
                j += 6;
            }
        }

        #[test]
        fn test_first_layer_bounding_box() {
            let bb = FIRST_LAYER.bounding_box();
            assert_eq!(bb.min.x, -3.0);
            assert_eq!(bb.min.y, -3.0);
            assert_eq!(bb.width(), 6.0);
            assert_eq!(bb.height(), 6.0);
        }
    }

    mod partial_layer {
        use super::*;

        pub const FIRST_LAYER_PARTIAL: ChunkCoords = ChunkCoords {
            cell_width: Length(1.0),
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
            let vertices = FIRST_LAYER_PARTIAL.positions(VertexSettings::default());
            assert_eq!(vertices.len(), 14);

            let radius = 3.0;
            let num_radial_lines = 12;
            println!("radius: {radius}");
            println!("num_radial_lines: {num_radial_lines}");
            assert_approx_eq_v2!(
                vertices[0],
                Vec2::new(
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[1],
                Vec2::new(
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[2],
                Vec2::new(
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[3],
                Vec2::new(
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[4],
                Vec2::new(
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[5],
                Vec2::new(
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[6],
                Vec2::new(
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );

            let radius = 4.0;
            assert_approx_eq_v2!(
                vertices[7],
                Vec2::new(
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -6.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[8],
                Vec2::new(
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -7.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[9],
                Vec2::new(
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -8.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[10],
                Vec2::new(
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -9.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[11],
                Vec2::new(
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -10.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[12],
                Vec2::new(
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -11.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
            assert_approx_eq_v2!(
                vertices[13],
                Vec2::new(
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).cos())
                        .approx()
                        .unwrap(),
                    (radius * (2.0 * PI * -12.0 / f64::from(num_radial_lines)).sin())
                        .approx()
                        .unwrap(),
                )
            );
        }

        #[test]
        fn test_first_layer_uv_partial() {
            let uvs = FIRST_LAYER_PARTIAL.uvs(VertexSettings::default());
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

    mod grid {
        mod core {

            use std::f64::consts::PI;

            use bevy::math::Vec2;

            use crate::physics::fallingsand::mesh::chunk_coords::tests::vec2_approx_eq;
            use crate::physics::fallingsand::mesh::chunk_coords::{
                ChunkCoords, VertexMode, VertexSettings,
            };
            use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
            use crate::physics::orbits::components::Length;

            pub const CORE: ChunkCoords = ChunkCoords {
                cell_width: Length(1.0),
                num_concentric_circles: 1,
                chunk_idx: ChunkIjkVector { i: 0, j: 0, k: 0 },
                start_concentric_circle_layer_relative: 0,
                start_radial_line: 0,
                end_radial_line: 12,
                layer_num_radial_lines: 12,
                start_concentric_circle_absolute: 0,
            };

            #[test]
            fn test_lod_1_pos() {
                let vertices = CORE.positions(VertexSettings {
                    lod: 1,
                    mode: VertexMode::Grid,
                });
                assert_eq!(vertices.len(), 26);

                // The core
                let radius: f64 = f64::from(CORE.end_radius());
                let diff_theta = 2.0 * PI / f64::from(CORE.num_radial_lines());
                assert_approx_eq_v2!(vertices[0], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[1], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[2], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[3], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[4], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[5], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[6], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[7], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[8], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[9], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[10], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[11], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[12], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[13], Vec2::new(radius as f32, 0.0));
                assert_approx_eq_v2!(
                    vertices[14],
                    Vec2::new(
                        (radius * diff_theta.cos()) as f32,
                        (-radius * diff_theta.sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[15],
                    Vec2::new(
                        (radius * (diff_theta * 2.0).cos()) as f32,
                        (-radius * (diff_theta * 2.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[16],
                    Vec2::new(
                        (radius * (diff_theta * 3.0).cos()) as f32,
                        (-radius * (diff_theta * 3.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[17],
                    Vec2::new(
                        (radius * (diff_theta * 4.0).cos()) as f32,
                        (-radius * (diff_theta * 4.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[18],
                    Vec2::new(
                        (radius * (diff_theta * 5.0).cos()) as f32,
                        (-radius * (diff_theta * 5.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[19],
                    Vec2::new(
                        (radius * (diff_theta * 6.0).cos()) as f32,
                        (-radius * (diff_theta * 6.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[20],
                    Vec2::new(
                        (radius * (diff_theta * 7.0).cos()) as f32,
                        (-radius * (diff_theta * 7.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[21],
                    Vec2::new(
                        (radius * (diff_theta * 8.0).cos()) as f32,
                        (-radius * (diff_theta * 8.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[22],
                    Vec2::new(
                        (radius * (diff_theta * 9.0).cos()) as f32,
                        (-radius * (diff_theta * 9.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[23],
                    Vec2::new(
                        (radius * (diff_theta * 10.0).cos()) as f32,
                        (-radius * (diff_theta * 10.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[24],
                    Vec2::new(
                        (radius * (diff_theta * 11.0).cos()) as f32,
                        (-radius * (diff_theta * 11.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[25],
                    Vec2::new(
                        (radius * (diff_theta * 12.0).cos()) as f32,
                        (-radius * (diff_theta * 12.0).sin()) as f32
                    )
                );
            }

            #[test]
            fn test_lod_2_pos() {
                let vertices = CORE.positions(VertexSettings {
                    lod: 2,
                    mode: VertexMode::Grid,
                });
                assert_eq!(vertices.len(), 14);

                // The core
                let radius = f64::from(CORE.end_radius());
                let diff_theta = 2.0 * PI / f64::from(CORE.num_radial_lines()) * 2.0;
                assert_approx_eq_v2!(vertices[0], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[1], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[2], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[3], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[4], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[5], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[6], Vec2::new(0.0, 0.0));
                assert_approx_eq_v2!(vertices[7], Vec2::new(radius as f32, 0.0));
                assert_approx_eq_v2!(
                    vertices[8],
                    Vec2::new(
                        (radius * diff_theta.cos()) as f32,
                        (-radius * diff_theta.sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[9],
                    Vec2::new(
                        (radius * (diff_theta * 2.0).cos()) as f32,
                        (-radius * (diff_theta * 2.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[10],
                    Vec2::new(
                        (radius * (diff_theta * 3.0).cos()) as f32,
                        (-radius * (diff_theta * 3.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[11],
                    Vec2::new(
                        (radius * (diff_theta * 4.0).cos()) as f32,
                        (-radius * (diff_theta * 4.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[12],
                    Vec2::new(
                        (radius * (diff_theta * 5.0).cos()) as f32,
                        (-radius * (diff_theta * 5.0).sin()) as f32
                    )
                );
                assert_approx_eq_v2!(
                    vertices[13],
                    Vec2::new(
                        (radius * (diff_theta * 6.0).cos()) as f32,
                        (-radius * (diff_theta * 6.0).sin()) as f32
                    )
                );
            }
        }
    }
}
