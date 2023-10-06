#![feature(core_panic)]

use super::chunk::Chunk;
use super::core::CoreChunk;
use super::partial_layer::{PartialLayerChunk, PartialLayerChunkBuilder};
use macroquad::prelude::Mesh;

pub struct RadialMesh {
    // cell_radius: f32,
    // num_layers: usize,
    // first_num_radial_lines: usize,
    // second_num_concentric_circles: usize,
    _core_chunk: CoreChunk,
    _partial_chunks: Vec<PartialLayerChunk>,
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
            max_cells: 1000,
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
        self.first_num_radial_lines = first_num_radial_lines;
        self
    }

    pub fn second_num_concentric_circles(mut self, second_num_concentric_circles: usize) -> Self {
        self.second_num_concentric_circles = second_num_concentric_circles;
        self
    }

    // pub fn max_cells(mut self, max_cells: usize) -> Self {
    //     self.max_cells = max_cells;
    //     self
    // }

    pub fn build(self) -> RadialMesh {
        if self.num_layers <= 1 {
            panic!("RadialMesh::new: num_layers must be greater than 1");
        }
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
            if next_layer.total_size() > self.max_cells {
                panic!(
                    "RadialMesh::new: next_layer.total_size() > self.max_cells: {} > {}",
                    next_layer.total_size(),
                    self.max_cells
                );
            }
            _partial_chunks.push(next_layer);

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, break
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
                if next_layer.total_size() > self.max_cells {
                    panic!("RadialMesh::new: i: {}/{} next_layer.total_size() > self.max_cells: {} > {}", i, num_radial_chunks, next_layer.total_size(), self.max_cells);
                }
                _partial_chunks.push(next_layer);
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, multiply the number of radial chunks by 2
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles)
                > self.max_cells
            {
                num_radial_chunks *= 2;
            }

            // If our width is smaller than our height, break
            if layer_num_radial_lines / num_radial_chunks < num_concentric_circles {
                break;
            }
        }

        // Handle the third set of layers, which just subdivide both around the grid and up/down the grid
        let mut num_concentric_chunks = 4;
        loop {
            if layer_num >= self.num_layers {
                break;
            }

            // TODO: Check this
            for i in 0..num_radial_chunks {
                for j in 0..num_concentric_chunks {
                    let next_layer = PartialLayerChunkBuilder::new()
                        .cell_radius(self.cell_radius)
                        .layer_num_radial_lines(layer_num_radial_lines)
                        .num_concentric_circles(num_concentric_circles / num_concentric_chunks)
                        .start_concentric_circle_absolute(start_concentric_circle_absolute)
                        .start_concentric_circle_layer_relative(
                            j * (num_concentric_circles / num_concentric_chunks),
                        )
                        .start_radial_line(i * (layer_num_radial_lines / num_radial_chunks))
                        .end_radial_line((i + 1) * (layer_num_radial_lines / num_radial_chunks))
                        .build();
                    if next_layer.total_size() > self.max_cells {
                        panic!("RadialMesh::new: i: {}/{} j: {}/{} next_layer.total_size() > self.max_cells: {} > {}", i, num_radial_chunks, j, num_concentric_chunks, next_layer.total_size(), self.max_cells);
                    }
                    _partial_chunks.push(next_layer);
                }
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, multiply the number of concentric chunks and radial chunks by 2
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles
                / num_concentric_chunks)
                > self.max_cells
            {
                num_radial_chunks *= 2;
                num_concentric_chunks *= 2;
            }
        }

        RadialMesh {
            // cell_radius: self.cell_radius,
            // num_layers: self.num_layers,
            // first_num_radial_lines: self.first_num_radial_lines,
            // second_num_concentric_circles: self.second_num_concentric_circles,
            _core_chunk,
            _partial_chunks,
        }
    }
}

impl RadialMesh {
    pub fn get_meshes(&self) -> Vec<Mesh> {
        let mut meshes: Vec<Mesh> = Vec::with_capacity(self._partial_chunks.len() + 1);
        meshes.push(self._core_chunk.get_mesh());
        for partial_chunk in self._partial_chunks.iter() {
            meshes.push(partial_chunk.get_mesh());
        }
        meshes
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

// }
