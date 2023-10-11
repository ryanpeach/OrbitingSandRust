use crate::physics::fallingsand::functions::{is_pow_2, ChunkIjkVector, Grid};

use super::chunk_coords::ChunkCoords;
use super::core_coords::CoreChunkCoords;
use super::layer_coords::{PartialLayerChunkCoords, PartialLayerChunkCoordsBuilder};
use crate::physics::fallingsand::functions::{MeshDrawMode, OwnedMeshData};
use ggez::glam::Vec2;
use ggez::graphics::{Rect, Vertex};

/// A structure that contains all the chunk coordinates for a celestial body
/// Useful for drawing the total mesh
#[derive(Clone)]
pub struct CoordinateDir {
    /// Every celestial body has a core
    /// The core is always the first layer and has 1 concentric circle
    core_chunk: CoreChunkCoords,

    /// Layers on top of the core
    /// Every index in the vec represents a layer
    /// The Grid then represents the chunks in that layer
    partial_chunks: Vec<Grid<PartialLayerChunkCoords>>,
}

/// A builder for CoordinateDir
/// Needs more parameters than CoordinateDir because
/// it assembles the chunks whereas CoordinateDir can re-derive
/// these parameters from the chunks themselves
pub struct CoordinateDirBuilder {
    cell_radius: f32,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    max_cells: usize,
}

/// Builds a CoordinateDir
/// This is where most of the logic is stored for assembling the directory
impl CoordinateDirBuilder {
    /// Start here
    pub fn new() -> Self {
        Self {
            cell_radius: 1.0,
            num_layers: 1,
            first_num_radial_lines: 6,
            second_num_concentric_circles: 2,
            max_cells: 64 * 64,
        }
    }
    /// The radius of each cell in the circle
    pub fn cell_radius(mut self, cell_radius: f32) -> Self {
        self.cell_radius = cell_radius;
        self
    }
    /// The number of layers in the circle
    pub fn num_layers(mut self, num_layers: usize) -> Self {
        self.num_layers = num_layers;
        self
    }
    /// The number of radial lines in the core.
    /// Each future layer has 2x the number of radial lines as the previous layer.
    pub fn first_num_radial_lines(mut self, first_num_radial_lines: usize) -> Self {
        self.first_num_radial_lines = first_num_radial_lines;
        self
    }
    /// The number of concentric circles in the second layer.
    /// Each future layer has 2x the number of concentric circles as the previous layer.
    /// The reason we define the second layer separately is because the core always has 1
    pub fn second_num_concentric_circles(mut self, second_num_concentric_circles: usize) -> Self {
        debug_assert!(
            is_pow_2(second_num_concentric_circles),
            "second_num_concentric_circles must be a power of 2, got {}",
            second_num_concentric_circles
        );
        self.second_num_concentric_circles = second_num_concentric_circles;
        self
    }
    /// The max number of cells a chunk is allowed to have
    /// If this number is reached, chunks split in half vertically and horizontally
    pub fn max_cells(mut self, max_cells: usize) -> Self {
        self.max_cells = max_cells;
        self
    }
    /// builds a CoordinateDir by iterating over the number of layers
    /// and dynamically allocating chunks to each layer based on max_cells
    /// and the other parameters of the builder.
    pub fn build(self) -> CoordinateDir {
        debug_assert_ne!(self.num_layers, 0);
        let _core_chunk = CoreChunkCoords::new(self.cell_radius, self.first_num_radial_lines);
        let mut _partial_chunks: Vec<PartialLayerChunkCoords> = Vec::new();

        // These variables will help us keep track of the current layer
        let mut layer_num_radial_lines = self.first_num_radial_lines * 2;
        let mut num_concentric_circles = self.second_num_concentric_circles;
        let mut start_concentric_circle_absolute = 1;
        let mut layer_num = 1;

        // Handle the first few layers
        loop {
            if layer_num >= self.num_layers {
                break;
            }
            let next_layer = PartialLayerChunkCoordsBuilder::new()
                .cell_radius(self.cell_radius)
                .layer_num(layer_num)
                .layer_num_radial_lines(layer_num_radial_lines)
                .num_concentric_circles(num_concentric_circles)
                .start_concentric_circle_absolute(start_concentric_circle_absolute)
                .start_concentric_circle_layer_relative(0)
                .start_radial_line(0)
                .end_radial_line(layer_num_radial_lines)
                .build();
            debug_assert!(next_layer.total_size() <= self.max_cells);
            _partial_chunks.push(next_layer);

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At self.max_cells, break
            if (layer_num_radial_lines * num_concentric_circles) > self.max_cells {
                break;
            }
        }

        // Handle the second set of layers, which just subdivide around the grid
        let mut num_radial_chunks = 4;
        loop {
            if layer_num >= self.num_layers {
                break;
            }

            // TODO: Check this
            for i in 0..num_radial_chunks {
                let next_layer = PartialLayerChunkCoordsBuilder::new()
                    .cell_radius(self.cell_radius)
                    .layer_num_radial_lines(layer_num_radial_lines)
                    .layer_num(layer_num)
                    .num_concentric_circles(num_concentric_circles)
                    .start_concentric_circle_absolute(start_concentric_circle_absolute)
                    .start_concentric_circle_layer_relative(0)
                    .start_radial_line(i * (layer_num_radial_lines / num_radial_chunks))
                    .end_radial_line((i + 1) * (layer_num_radial_lines / num_radial_chunks))
                    .build();
                debug_assert!(next_layer.total_size() <= self.max_cells);
                _partial_chunks.push(next_layer);
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // If our width would become smaller than our height, break
            if layer_num_radial_lines / (num_radial_chunks * 4) < num_concentric_circles {
                break;
            }

            // At self.max_cells, multiply the number of radial chunks by 4
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles)
                > self.max_cells
            {
                num_radial_chunks *= 4;
            }
        }

        // Handle the third set of layers, which just subdivide both around the grid and up/down the grid
        let mut num_concentric_chunks = 2;
        num_radial_chunks *= 2;
        loop {
            if layer_num >= self.num_layers {
                break;
            }

            // TODO: Check this
            for j in 0..num_concentric_chunks {
                for k in 0..num_radial_chunks {
                    let next_layer = PartialLayerChunkCoordsBuilder::new()
                        .cell_radius(self.cell_radius)
                        .layer_num_radial_lines(layer_num_radial_lines)
                        .layer_num(layer_num)
                        .num_concentric_circles(num_concentric_circles / num_concentric_chunks)
                        .start_concentric_circle_absolute(start_concentric_circle_absolute)
                        .start_concentric_circle_layer_relative(
                            j * (num_concentric_circles / num_concentric_chunks),
                        )
                        .start_radial_line(k * (layer_num_radial_lines / num_radial_chunks))
                        .end_radial_line((k + 1) * (layer_num_radial_lines / num_radial_chunks))
                        .build();
                    debug_assert!(next_layer.total_size() <= self.max_cells);
                    _partial_chunks.push(next_layer);
                }
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At self.max_cells, multiply the number of concentric chunks and radial chunks by 2
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles
                / num_concentric_chunks)
                > self.max_cells
            {
                num_radial_chunks *= 2;
                num_concentric_chunks *= 2;
            }
        }

        CoordinateDir {
            core_chunk: _core_chunk,
            partial_chunks: _partial_chunks,
        }
    }
}

/* =========================================
 *           Aggregate Getters
 * These functions run a getter over each
 * chunk and return a vector of the results
 * ========================================= */
impl CoordinateDir {
    pub fn get_outlines(&self) -> Vec<Vec<Vec2>> {
        let mut outlines = Vec::new();
        outlines.push(self.core_chunk.get_outline());
        for layer in &self.partial_chunks {
            for partial_chunk in layer.get_data() {
                outlines.push(partial_chunk.get_outline());
            }
        }
        outlines
    }
    pub fn get_vertexes(&self, res: u16) -> Vec<Vec<Vertex>> {
        let mut vertexes = Vec::new();
        vertexes.push(self.core_chunk.get_vertices(res));
        for partial_chunk in &self.partial_chunks {
            vertexes.push(partial_chunk.get_vertices(res));
        }
        vertexes
    }
    pub fn get_positions(&self, res: u16) -> Vec<Vec<Vec2>> {
        let mut positions = Vec::new();
        positions.push(self.core_chunk.get_positions(res));
        for partial_chunk in &self.partial_chunks {
            positions.push(partial_chunk.get_positions(res));
        }
        positions
    }
    pub fn get_uvs(&self, res: u16) -> Vec<Vec<Vec2>> {
        let mut uvs = Vec::new();
        uvs.push(self.core_chunk.get_uvs(res));
        for partial_chunk in &self.partial_chunks {
            uvs.push(partial_chunk.get_uvs(res));
        }
        uvs
    }
    pub fn get_indices(&self, res: u16) -> Vec<Vec<u32>> {
        let mut indices = Vec::new();
        indices.push(self.core_chunk.get_indices(res));
        for partial_chunk in &self.partial_chunks {
            indices.push(partial_chunk.get_indices(res));
        }
        indices
    }
    pub fn get_chunk_bounding_boxes(&self) -> Vec<Rect> {
        let mut bounding_boxes = Vec::new();
        bounding_boxes.push(self.core_chunk.get_bounding_box());
        for partial_chunk in &self.partial_chunks {
            bounding_boxes.push(partial_chunk.get_bounding_box());
        }
        bounding_boxes
    }
}

/* =========================================
 *         Individual Chunk Getters
 * These functions run a getter on a specific
 * chunk index
 * ========================================= */
impl CoordinateDir {
    pub fn get_chunk_at_idx(&self, chunk_idx: ChunkIjkVector) -> Box<dyn ChunkCoords> {
        if chunk_idx == 0 {
            Box::new(self.core_chunk)
        } else {
            Box::new(self.partial_chunks[chunk_idx - 1])
        }
    }
    pub fn get_chunk_bounding_box(&self, chunk_idx: ChunkIjkVector) -> Rect {
        if chunk_idx == 0 {
            self.core_chunk.get_bounding_box()
        } else {
            self.partial_chunks[chunk_idx - 1].get_bounding_box()
        }
    }
    pub fn get_chunk_start_radius(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_start_radius()
        } else {
            self.partial_chunks[chunk_idx - 1].get_start_radius()
        }
    }
    pub fn get_chunk_end_radius(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_end_radius()
        } else {
            self.partial_chunks[chunk_idx - 1].get_end_radius()
        }
    }
    pub fn get_chunk_start_radial_theta(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_start_radial_theta()
        } else {
            self.partial_chunks[chunk_idx - 1].get_start_radial_theta()
        }
    }
    pub fn get_chunk_end_radial_theta(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_end_radial_theta()
        } else {
            self.partial_chunks[chunk_idx - 1].get_end_radial_theta()
        }
    }
    pub fn get_chunk_num_radial_lines(&self, chunk_idx: ChunkIjkVector) -> usize {
        if chunk_idx == 0 {
            self.core_chunk.get_num_radial_lines()
        } else {
            self.partial_chunks[chunk_idx - 1].get_num_radial_lines()
        }
    }
    pub fn get_chunk_num_concentric_circles(&self, chunk_idx: ChunkIjkVector) -> usize {
        if chunk_idx == 0 {
            self.core_chunk.get_num_concentric_circles()
        } else {
            self.partial_chunks[chunk_idx - 1].get_num_concentric_circles()
        }
    }
}

/* ========================================
 * Simple Getters
 * Misc attributes of the directory itself.
 * ======================================== */
impl CoordinateDir {
    /// The total number of cells in the whole directory
    pub fn total_size(&self) -> usize {
        let mut total_size = self.core_chunk.total_size();
        for partial_chunk in &self.partial_chunks {
            total_size += partial_chunk.total_size();
        }
        total_size
    }
    /// Cell radius is constant for all chunks
    pub fn get_cell_radius(&self) -> f32 {
        self.core_chunk.get_cell_radius()
    }
    /// The number of layers in the circle
    pub fn get_num_layers(&self) -> usize {
        self.partial_chunks.len() + 1
    }
    /// The number of concentric circles in a given layer
    /// Always 2x the previous layer except for the first and second layers
    pub fn get_layer_num_concentric_circles(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            self.core_chunk.get_num_concentric_circles()
        } else {
            self.partial_chunks[layer_num - 1].get_num_concentric_circles()
        }
    }
    /// The number of radial lines in a given layer
    /// Always 2x the previous layer except for the first layer
    pub fn get_layer_num_radial_lines(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            self.core_chunk.get_num_radial_lines()
        } else {
            self.partial_chunks[layer_num - 1].get_num_radial_lines()
        }
    }
    /// The total number of chunks in the directory
    pub fn get_num_chunks(&self) -> usize {
        self.partial_chunks.len() + 1
    }
}

/* ===================
 * Drawing
 * =================== */
impl CoordinateDir {
    /// Gets mesh data for every chunk in the directory
    pub fn get_mesh_data(&self, res: u16, draw_mode: MeshDrawMode) -> Vec<OwnedMeshData> {
        (0..self.get_num_chunks())
            .map(|chunk_idx| {
                if chunk_idx == 0 {
                    match draw_mode {
                        MeshDrawMode::TexturedMesh => self.core_chunk.calc_chunk_meshdata(res),
                        MeshDrawMode::UVWireframe => self.core_chunk.calc_chunk_uv_wireframe(res),
                        MeshDrawMode::TriangleWireframe => {
                            self.core_chunk.calc_chunk_triangle_wireframe(res)
                        }
                        MeshDrawMode::Outline => self.core_chunk.calc_chunk_outline(),
                    }
                } else {
                    match draw_mode {
                        MeshDrawMode::TexturedMesh => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_meshdata(res)
                        }
                        MeshDrawMode::UVWireframe => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_uv_wireframe(res)
                        }
                        MeshDrawMode::TriangleWireframe => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_triangle_wireframe(res)
                        }
                        MeshDrawMode::Outline => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_outline()
                        }
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::physics::fallingsand::functions::valid_step;

    use super::*;

    #[test]
    fn test_radial_mesh_chunk_sizes() {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(8)
            .second_num_concentric_circles(2)
            .max_cells(576) // 24x24
            .build();

        // Check that for all resolutions 2^0 to 2^6, the chunk sizes are valid for grid_iter
        // In most of our methods we iterate over the +1 of the dimension sizes, so we add one to each
        for chunk_num in 0..coordinate_dir.get_num_chunks() {
            for i in 0..7 {
                assert!(
                    valid_step(
                        coordinate_dir.get_chunk_num_concentric_circles(chunk_num) + 1,
                        2usize.pow(i as u32)
                    ),
                    "layer {} concentric circles + 1 is not valid at a step of {}",
                    chunk_num,
                    2usize.pow(i as u32)
                );
                assert!(
                    valid_step(
                        coordinate_dir.get_chunk_num_radial_lines(chunk_num) + 1,
                        2usize.pow(i as u32)
                    ),
                    "layer {} radial lines + 1 is not valid at a step of {}",
                    chunk_num,
                    2usize.pow(i as u32)
                );
            }
        }
    }

    #[test]
    fn test_radial_mesh_chunk_sizes_manual() {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(8)
            .second_num_concentric_circles(2)
            .max_cells(576) // 24x24
            .build();

        // Layer 0
        // Test that the first chunk is 1x8
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(0), 8);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(0), 1);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 0 x 1 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(0), 0.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(0), 1.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(0), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(0),
            2.0 * std::f32::consts::PI
        );

        // Layer 1
        // Test that the next chunk is 2x16
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(1), 16);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(1), 2);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 1 x 3 x 0 x 2pi
        // 1 comes from the previous layer's end_radius
        // 3 comes from the previous layer's end_radius + the previous layers (end_radius - start_radius)*2
        assert_eq!(coordinate_dir.get_chunk_start_radius(1), 1.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(1), 3.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(1), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(1),
            2.0 * std::f32::consts::PI
        );

        // Layer 2
        // Test that the next chunk is 4x32
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(2), 32);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(2), 4);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 3 x 7 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(2), 3.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(2), 7.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(2), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(2),
            2.0 * std::f32::consts::PI
        );

        // Layer 3
        // Test that the next chunk is 8x64
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(3), 64);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(3), 8);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 7 x 15 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(3), 7.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(3), 15.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(3), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(3),
            2.0 * std::f32::consts::PI
        );

        // Layer 4
        // Now we have split in 4 because 16x128 is 2048, which is bigger than 576
        // Test that the next 4 chunks are 16x32
        // From now on the sizes are stable
        for i in 0..4 {
            assert_eq!(coordinate_dir.get_chunk_num_radial_lines(4 + i), 32);
            assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(4 + i), 16);
        }
        // I want to test the start_radius x end_radius x start_radial_theta x end_radial_theta by hand
        // The first one will be 15 x 31 x 0*2pi/4 x 1*2pi/4
        // The second one will be 15 x 31 x 1*2pi/4 x 2*2pi/4
        // The third one will be 15 x 31 x 2*2pi/4 x 3*2pi/4
        // The fourth one will be 15 x 31 x 3*2pi/4 x 4*2pi/4
        // Making a macro for convienience, also this will keep the resulting error on the right line
        macro_rules! assert_quad {
            ($coordinate_dir:expr, $chunk_idx:expr, $start_radius:expr, $end_radius:expr, $start_radial_theta:expr, $end_radial_theta:expr) => {
                assert_eq!(
                    $coordinate_dir.get_chunk_start_radius($chunk_idx),
                    $start_radius,
                    "start_radius is incorrect."
                );
                assert_eq!(
                    $coordinate_dir.get_chunk_end_radius($chunk_idx),
                    $end_radius,
                    "end_radius is incorrect."
                );
                assert_eq!(
                    $coordinate_dir.get_chunk_start_radial_theta($chunk_idx),
                    $start_radial_theta,
                    "start_radial_theta is incorrect."
                );
                assert_eq!(
                    $coordinate_dir.get_chunk_end_radial_theta($chunk_idx),
                    $end_radial_theta,
                    "end_radial_theta is incorrect."
                );
            };
        }
        assert_quad!(
            coordinate_dir,
            4,
            15.0,
            31.0,
            0.0 * 2.0 * std::f32::consts::PI / 4.0,
            1.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            coordinate_dir,
            5,
            15.0,
            31.0,
            1.0 * 2.0 * std::f32::consts::PI / 4.0,
            2.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            coordinate_dir,
            6,
            15.0,
            31.0,
            2.0 * 2.0 * std::f32::consts::PI / 4.0,
            3.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            coordinate_dir,
            7,
            15.0,
            31.0,
            3.0 * 2.0 * std::f32::consts::PI / 4.0,
            4.0 * 2.0 * std::f32::consts::PI / 4.0
        );

        // Layer 5
        // This layer is 32x256
        // We split by 2 again in the radial direction, meaning we are split by 8 in the radial direction
        // And we split by 2 in the concentric direction
        // This means the next 16 chunks would be 32x16
        for i in 0..16 {
            assert_eq!(coordinate_dir.get_chunk_num_radial_lines(8 + i), 32);
            assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(8 + i), 16);
        }
        // So the first 8 chunks are in a concentric circle
        assert_quad!(
            coordinate_dir,
            8,
            31.0,
            47.0,
            0.0 * 2.0 * std::f32::consts::PI / 8.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            9,
            31.0,
            47.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            10,
            31.0,
            47.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            11,
            31.0,
            47.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            12,
            31.0,
            47.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            13,
            31.0,
            47.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            14,
            31.0,
            47.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            15,
            31.0,
            47.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0,
            8.0 * 2.0 * std::f32::consts::PI / 8.0
        );

        // Then the next 8 chunks are in the next concentric circle
        assert_quad!(
            coordinate_dir,
            16,
            47.0,
            63.0,
            0.0 * 2.0 * std::f32::consts::PI / 8.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            17,
            47.0,
            63.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            18,
            47.0,
            63.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            19,
            47.0,
            63.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            20,
            47.0,
            63.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            21,
            47.0,
            63.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            22,
            47.0,
            63.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            coordinate_dir,
            23,
            47.0,
            63.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0,
            8.0 * 2.0 * std::f32::consts::PI / 8.0
        );

        // Layer 6
        // From now on the layer size should be stable
        for i in 0..64 {
            assert_eq!(coordinate_dir.get_chunk_num_radial_lines(24 + i), 32);
            assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(24 + i), 16);
        }
    }
}
