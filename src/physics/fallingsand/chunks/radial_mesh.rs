use crate::physics::fallingsand::chunks::util::is_pow_2;

use super::chunk::Chunk;
use super::core::CoreChunk;
use super::partial_layer::{PartialLayerChunk, PartialLayerChunkBuilder};
use super::util::{DrawMode, OwnedMeshData, RawImage};
use ggez::glam::Vec2;
use ggez::graphics::{Rect, Vertex};

pub struct RadialMesh {
    num_layers: usize,
    core_chunk: CoreChunk,
    partial_chunks: Vec<PartialLayerChunk>,
}

pub struct RadialMeshBuilder {
    cell_radius: f32,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    max_cells: usize,
}

impl RadialMeshBuilder {
    pub fn new() -> Self {
        Self {
            cell_radius: 1.0,
            num_layers: 1,
            first_num_radial_lines: 1,
            second_num_concentric_circles: 1,
            max_cells: 64 * 64,
        }
    }

    pub fn cell_radius(mut self, cell_radius: f32) -> Self {
        self.cell_radius = cell_radius;
        self
    }

    pub fn num_layers(mut self, num_layers: usize) -> Self {
        self.num_layers = num_layers;
        self
    }

    pub fn first_num_radial_lines(mut self, first_num_radial_lines: usize) -> Self {
        debug_assert!(
            is_pow_2(first_num_radial_lines),
            "first_num_radial_lines must be a power of 2, got {}",
            first_num_radial_lines
        );
        self.first_num_radial_lines = first_num_radial_lines;
        self
    }

    pub fn second_num_concentric_circles(mut self, second_num_concentric_circles: usize) -> Self {
        debug_assert!(
            is_pow_2(second_num_concentric_circles),
            "second_num_concentric_circles must be a power of 2, got {}",
            second_num_concentric_circles
        );
        self.second_num_concentric_circles = second_num_concentric_circles;
        self
    }

    pub fn max_cells(mut self, max_cells: usize) -> Self {
        self.max_cells = max_cells;
        self
    }

    pub fn build(self) -> RadialMesh {
        debug_assert_ne!(self.num_layers, 0);
        let _core_chunk = CoreChunk::new(self.cell_radius, self.first_num_radial_lines);
        let mut _partial_chunks: Vec<PartialLayerChunk> = Vec::new();

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
            let next_layer = PartialLayerChunkBuilder::new()
                .cell_radius(self.cell_radius)
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
                let next_layer = PartialLayerChunkBuilder::new()
                    .cell_radius(self.cell_radius)
                    .layer_num_radial_lines(layer_num_radial_lines)
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
                    let next_layer = PartialLayerChunkBuilder::new()
                        .cell_radius(self.cell_radius)
                        .layer_num_radial_lines(layer_num_radial_lines)
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

        RadialMesh {
            num_layers: self.num_layers,
            core_chunk: _core_chunk,
            partial_chunks: _partial_chunks,
        }
    }
}

impl RadialMesh {
    pub fn get_outlines(&self, res: u16) -> Vec<Vec<Vec2>> {
        let mut outlines = Vec::new();
        outlines.push(self.core_chunk.get_outline(res));
        for partial_chunk in &self.partial_chunks {
            outlines.push(partial_chunk.get_outline(res));
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

    pub fn get_textures(&self, res: u16) -> Vec<RawImage> {
        let mut textures = Vec::new();
        textures.push(self.core_chunk.get_texture(res));
        for partial_chunk in &self.partial_chunks {
            textures.push(partial_chunk.get_texture(res));
        }
        textures
    }

    pub fn get_texture(&self, res: u16, chunk_idx: usize) -> RawImage {
        if chunk_idx == 0 {
            self.core_chunk.get_texture(res)
        } else {
            self.partial_chunks[chunk_idx - 1].get_texture(res)
        }
    }
    pub fn get_chunk_bounding_box(&self, chunk_idx: usize) -> Rect {
        if chunk_idx == 0 {
            self.core_chunk.get_bounding_box()
        } else {
            self.partial_chunks[chunk_idx - 1].get_bounding_box()
        }
    }
    pub fn get_chunk_bounding_boxes(&self) -> Vec<Rect> {
        let mut bounding_boxes = Vec::new();
        bounding_boxes.push(self.core_chunk.get_bounding_box());
        for partial_chunk in &self.partial_chunks {
            bounding_boxes.push(partial_chunk.get_bounding_box());
        }
        bounding_boxes
    }
    pub fn get_mesh_data(&self, res: u16, draw_mode: DrawMode) -> Vec<OwnedMeshData> {
        (0..self.get_num_chunks())
            .map(|chunk_idx| {
                if chunk_idx == 0 {
                    match draw_mode {
                        DrawMode::TexturedMesh => self.core_chunk.calc_chunk_meshdata(res),
                        DrawMode::UVWireframe => self.core_chunk.calc_chunk_uv_wireframe(res),
                        DrawMode::TriangleWireframe => {
                            self.core_chunk.calc_chunk_triangle_wireframe(res)
                        }
                    }
                } else {
                    match draw_mode {
                        DrawMode::TexturedMesh => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_meshdata(res)
                        }
                        DrawMode::UVWireframe => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_uv_wireframe(res)
                        }
                        DrawMode::TriangleWireframe => {
                            self.partial_chunks[chunk_idx - 1].calc_chunk_triangle_wireframe(res)
                        }
                    }
                }
            })
            .collect()
    }

    /* Shape Parameter Getters */
    pub fn get_cell_radius(&self) -> f32 {
        self.core_chunk.get_cell_radius()
    }
    pub fn get_num_layers(&self) -> usize {
        self.num_layers
    }
    pub fn get_num_chunks(&self) -> usize {
        self.partial_chunks.len() + 1
    }
    pub fn get_chunk_start_radius(&self, chunk_idx: usize) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_start_radius()
        } else {
            self.partial_chunks[chunk_idx - 1].get_start_radius()
        }
    }
    pub fn get_chunk_end_radius(&self, chunk_idx: usize) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_end_radius()
        } else {
            self.partial_chunks[chunk_idx - 1].get_end_radius()
        }
    }
    pub fn get_chunk_start_radial_theta(&self, chunk_idx: usize) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_start_radial_theta()
        } else {
            self.partial_chunks[chunk_idx - 1].get_start_radial_theta()
        }
    }
    pub fn get_chunk_end_radial_theta(&self, chunk_idx: usize) -> f32 {
        if chunk_idx == 0 {
            self.core_chunk.get_end_radial_theta()
        } else {
            self.partial_chunks[chunk_idx - 1].get_end_radial_theta()
        }
    }
    pub fn get_chunk_num_radial_lines(&self, chunk_idx: usize) -> usize {
        if chunk_idx == 0 {
            self.core_chunk.get_num_radial_lines()
        } else {
            self.partial_chunks[chunk_idx - 1].get_num_radial_lines()
        }
    }
    pub fn get_chunk_num_concentric_circles(&self, chunk_idx: usize) -> usize {
        if chunk_idx == 0 {
            self.core_chunk.get_num_concentric_circles()
        } else {
            self.partial_chunks[chunk_idx - 1].get_num_concentric_circles()
        }
    }
    pub fn total_size(&self) -> usize {
        let mut total_size = self.core_chunk.total_size();
        for partial_chunk in &self.partial_chunks {
            total_size += partial_chunk.total_size();
        }
        total_size
    }
}

#[cfg(test)]
mod tests {
    use crate::physics::fallingsand::chunks::util::valid_step;

    use super::*;

    #[test]
    fn test_radial_mesh_chunk_sizes() {
        let radial_mesh = RadialMeshBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(8)
            .second_num_concentric_circles(2)
            .max_cells(576) // 24x24
            .build();

        // Check that for all resolutions 2^0 to 2^6, the chunk sizes are valid for grid_iter
        // In most of our methods we iterate over the +1 of the dimension sizes, so we add one to each
        for chunk_num in 0..radial_mesh.get_num_chunks() {
            for i in 0..7 {
                assert!(
                    valid_step(
                        radial_mesh.get_chunk_num_concentric_circles(chunk_num) + 1,
                        2usize.pow(i as u32)
                    ),
                    "layer {} concentric circles + 1 is not valid at a step of {}",
                    chunk_num,
                    2usize.pow(i as u32)
                );
                assert!(
                    valid_step(
                        radial_mesh.get_chunk_num_radial_lines(chunk_num) + 1,
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
        let radial_mesh = RadialMeshBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(8)
            .second_num_concentric_circles(2)
            .max_cells(576) // 24x24
            .build();

        // Layer 0
        // Test that the first chunk is 1x8
        assert_eq!(radial_mesh.get_chunk_num_radial_lines(0), 8);
        assert_eq!(radial_mesh.get_chunk_num_concentric_circles(0), 1);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 0 x 1 x 0 x 2pi
        assert_eq!(radial_mesh.get_chunk_start_radius(0), 0.0);
        assert_eq!(radial_mesh.get_chunk_end_radius(0), 1.0);
        assert_eq!(radial_mesh.get_chunk_start_radial_theta(0), 0.0);
        assert_eq!(
            radial_mesh.get_chunk_end_radial_theta(0),
            2.0 * std::f32::consts::PI
        );

        // Layer 1
        // Test that the next chunk is 2x16
        assert_eq!(radial_mesh.get_chunk_num_radial_lines(1), 16);
        assert_eq!(radial_mesh.get_chunk_num_concentric_circles(1), 2);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 1 x 3 x 0 x 2pi
        // 1 comes from the previous layer's end_radius
        // 3 comes from the previous layer's end_radius + the previous layers (end_radius - start_radius)*2
        assert_eq!(radial_mesh.get_chunk_start_radius(1), 1.0);
        assert_eq!(radial_mesh.get_chunk_end_radius(1), 3.0);
        assert_eq!(radial_mesh.get_chunk_start_radial_theta(1), 0.0);
        assert_eq!(
            radial_mesh.get_chunk_end_radial_theta(1),
            2.0 * std::f32::consts::PI
        );

        // Layer 2
        // Test that the next chunk is 4x32
        assert_eq!(radial_mesh.get_chunk_num_radial_lines(2), 32);
        assert_eq!(radial_mesh.get_chunk_num_concentric_circles(2), 4);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 3 x 7 x 0 x 2pi
        assert_eq!(radial_mesh.get_chunk_start_radius(2), 3.0);
        assert_eq!(radial_mesh.get_chunk_end_radius(2), 7.0);
        assert_eq!(radial_mesh.get_chunk_start_radial_theta(2), 0.0);
        assert_eq!(
            radial_mesh.get_chunk_end_radial_theta(2),
            2.0 * std::f32::consts::PI
        );

        // Layer 3
        // Test that the next chunk is 8x64
        assert_eq!(radial_mesh.get_chunk_num_radial_lines(3), 64);
        assert_eq!(radial_mesh.get_chunk_num_concentric_circles(3), 8);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 7 x 15 x 0 x 2pi
        assert_eq!(radial_mesh.get_chunk_start_radius(3), 7.0);
        assert_eq!(radial_mesh.get_chunk_end_radius(3), 15.0);
        assert_eq!(radial_mesh.get_chunk_start_radial_theta(3), 0.0);
        assert_eq!(
            radial_mesh.get_chunk_end_radial_theta(3),
            2.0 * std::f32::consts::PI
        );

        // Layer 4
        // Now we have split in 4 because 16x128 is 2048, which is bigger than 576
        // Test that the next 4 chunks are 16x32
        // From now on the sizes are stable
        for i in 0..4 {
            assert_eq!(radial_mesh.get_chunk_num_radial_lines(4 + i), 32);
            assert_eq!(radial_mesh.get_chunk_num_concentric_circles(4 + i), 16);
        }
        // I want to test the start_radius x end_radius x start_radial_theta x end_radial_theta by hand
        // The first one will be 15 x 31 x 0*2pi/4 x 1*2pi/4
        // The second one will be 15 x 31 x 1*2pi/4 x 2*2pi/4
        // The third one will be 15 x 31 x 2*2pi/4 x 3*2pi/4
        // The fourth one will be 15 x 31 x 3*2pi/4 x 4*2pi/4
        // Making a macro for convienience, also this will keep the resulting error on the right line
        macro_rules! assert_quad {
            ($radial_mesh:expr, $chunk_idx:expr, $start_radius:expr, $end_radius:expr, $start_radial_theta:expr, $end_radial_theta:expr) => {
                assert_eq!(
                    $radial_mesh.get_chunk_start_radius($chunk_idx),
                    $start_radius,
                    "start_radius is incorrect."
                );
                assert_eq!(
                    $radial_mesh.get_chunk_end_radius($chunk_idx),
                    $end_radius,
                    "end_radius is incorrect."
                );
                assert_eq!(
                    $radial_mesh.get_chunk_start_radial_theta($chunk_idx),
                    $start_radial_theta,
                    "start_radial_theta is incorrect."
                );
                assert_eq!(
                    $radial_mesh.get_chunk_end_radial_theta($chunk_idx),
                    $end_radial_theta,
                    "end_radial_theta is incorrect."
                );
            };
        }
        assert_quad!(
            radial_mesh,
            4,
            15.0,
            31.0,
            0.0 * 2.0 * std::f32::consts::PI / 4.0,
            1.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            radial_mesh,
            5,
            15.0,
            31.0,
            1.0 * 2.0 * std::f32::consts::PI / 4.0,
            2.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            radial_mesh,
            6,
            15.0,
            31.0,
            2.0 * 2.0 * std::f32::consts::PI / 4.0,
            3.0 * 2.0 * std::f32::consts::PI / 4.0
        );
        assert_quad!(
            radial_mesh,
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
            assert_eq!(radial_mesh.get_chunk_num_radial_lines(8 + i), 32);
            assert_eq!(radial_mesh.get_chunk_num_concentric_circles(8 + i), 16);
        }
        // So the first 8 chunks are in a concentric circle
        assert_quad!(
            radial_mesh,
            8,
            31.0,
            47.0,
            0.0 * 2.0 * std::f32::consts::PI / 8.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            9,
            31.0,
            47.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            10,
            31.0,
            47.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            11,
            31.0,
            47.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            12,
            31.0,
            47.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            13,
            31.0,
            47.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            14,
            31.0,
            47.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            15,
            31.0,
            47.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0,
            8.0 * 2.0 * std::f32::consts::PI / 8.0
        );

        // Then the next 8 chunks are in the next concentric circle
        assert_quad!(
            radial_mesh,
            16,
            47.0,
            63.0,
            0.0 * 2.0 * std::f32::consts::PI / 8.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            17,
            47.0,
            63.0,
            1.0 * 2.0 * std::f32::consts::PI / 8.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            18,
            47.0,
            63.0,
            2.0 * 2.0 * std::f32::consts::PI / 8.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            19,
            47.0,
            63.0,
            3.0 * 2.0 * std::f32::consts::PI / 8.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            20,
            47.0,
            63.0,
            4.0 * 2.0 * std::f32::consts::PI / 8.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            21,
            47.0,
            63.0,
            5.0 * 2.0 * std::f32::consts::PI / 8.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            22,
            47.0,
            63.0,
            6.0 * 2.0 * std::f32::consts::PI / 8.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0
        );
        assert_quad!(
            radial_mesh,
            23,
            47.0,
            63.0,
            7.0 * 2.0 * std::f32::consts::PI / 8.0,
            8.0 * 2.0 * std::f32::consts::PI / 8.0
        );

        // Layer 6
        // From now on the layer size should be stable
        for i in 0..64 {
            assert_eq!(radial_mesh.get_chunk_num_radial_lines(24 + i), 32);
            assert_eq!(radial_mesh.get_chunk_num_concentric_circles(24 + i), 16);
        }
    }
}
