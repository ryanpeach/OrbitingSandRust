//! A simple Earth-like celestial body

use bevy::log::info;

use crate::bevy::entities::celestials::celestial::Data;
use crate::common::util::vectors::ChunkIjkVector;
use crate::{
    physics::{
        fallingsand::{
            data::element_directory::ElementGridDir, elements::element::ElementType,
            mesh::coordinate_dir, 
        },
        orbits::components::Length,
    },
};

/// Builds an earthlike celestial body
pub struct Builder {
    /// See [`coordinate_dir::Builder::cell_width`]
    cell_width: Length,
    /// See [`coordinate_dir::Builder::num_layers`]
    num_layers: u32,
    /// See [`coordinate_dir::Builder::first_num_radial_lines`]
    first_num_radial_lines: u32,
    /// See [`coordinate_dir::Builder::second_num_concentric_circles`]
    second_num_concentric_circles: u32,
    /// See [`coordinate_dir::Builder::first_num_tangential_chunks`]
    first_num_tangential_chunks: u32,
    /// See [`coordinate_dir::Builder::max_radial_lines_per_chunk`]
    max_radial_lines_per_chunk: u32,
    /// See [`coordinate_dir::Builder::max_concentric_circles_per_chunk`]
    max_concentric_circles_per_chunk: u32,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Create a new [`Builder`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            cell_width: Length(1.0),
            num_layers: 8,
            first_num_radial_lines: 12,
            second_num_concentric_circles: 3,
            first_num_tangential_chunks: 3,
            max_radial_lines_per_chunk: 128,
            max_concentric_circles_per_chunk: 128,
        }
    }

    /// Set [`Builder::cell_width`]
    #[must_use]
    pub fn cell_width(mut self, cell_width: Length) -> Self {
        self.cell_width = cell_width;
        self
    }

    /// Set [`Builder::num_layers`]
    #[must_use]
    pub fn num_layers(mut self, num_layers: u32) -> Self {
        self.num_layers = num_layers;
        self
    }

    /// Set [`Builder::first_num_radial_lines`]
    #[must_use]
    pub fn first_num_radial_lines(mut self, first_num_radial_lines: u32) -> Self {
        self.first_num_radial_lines = first_num_radial_lines;
        self
    }

    /// Set [`Builder::second_num_concentric_circles`]
    #[must_use]
    pub fn second_num_concentric_circles(mut self, second_num_concentric_circles: u32) -> Self {
        self.second_num_concentric_circles = second_num_concentric_circles;
        self
    }

    /// Set [`Builder::first_num_tangential_chunks`]
    #[must_use]
    pub fn first_num_tangential_chunkss(mut self, first_num_tangential_chunkss: u32) -> Self {
        self.first_num_tangential_chunks = first_num_tangential_chunkss;
        self
    }

    /// Set [`Builder::max_radial_lines_per_chunk`]
    #[must_use]
    pub fn max_radial_lines_per_chunk(mut self, max_radial_lines_per_chunk: u32) -> Self {
        self.max_radial_lines_per_chunk = max_radial_lines_per_chunk;
        self
    }

    /// Set [`Builder::max_concentric_circles_per_chunk`]
    #[must_use]
    pub fn max_concentric_circles_per_chunk(
        mut self,
        max_concentric_circles_per_chunk: u32,
    ) -> Self {
        self.max_concentric_circles_per_chunk = max_concentric_circles_per_chunk;
        self
    }

    /// Build [`Data`]
    pub fn build(&self) -> Data {
        let coordinate_dir = coordinate_dir::Builder::new()
            .cell_width(self.cell_width)
            .num_layers(self.num_layers)
            .first_num_radial_lines(self.first_num_radial_lines)
            .second_num_concentric_circles(self.second_num_concentric_circles)
            .first_num_tangential_chunkss(self.first_num_tangential_chunks)
            .max_radial_lines_per_chunk(self.max_radial_lines_per_chunk)
            .max_concentric_circles_per_chunk(self.max_concentric_circles_per_chunk)
            .build();
        let mut element_grid_dir = ElementGridDir::new_empty(coordinate_dir);
        info!("Num elements: {}", element_grid_dir.total_num_cells());

        // Iterate over each layer of the element grid and fill it with the appropriate element
        let mut total_j = 0;
        for layer_num in 0..element_grid_dir.coordinate_dir().num_layers() {
            for j in 0..element_grid_dir
                .coordinate_dir()
                .layer_num_concentric_chunks(layer_num)
            {
                for k in 0..element_grid_dir
                    .coordinate_dir()
                    .layer_num_tangential_chunkss(layer_num)
                {
                    let chunk_idx = ChunkIjkVector::new(layer_num, j, k);
                    let element_grid = element_grid_dir.chunk_at_chunk_ijk_mut(chunk_idx);
                    match total_j {
                        0..=3 => {
                            element_grid.fill(ElementType::Lava);
                        }
                        4..=9 => {
                            element_grid.fill(ElementType::Stone);
                        }
                        10..=12 => {
                            element_grid.fill(ElementType::Sand);
                        }
                        13..=14 => {
                            element_grid.fill(ElementType::Water);
                        }
                        15..=16 => {
                            if k % 2 == 0 {
                                element_grid.fill(ElementType::Vacuum);
                            } else {
                                element_grid.fill(ElementType::Sand);
                            }
                        }
                        _ => {
                            element_grid.fill(ElementType::Vacuum);
                        }
                    }
                }
                total_j += 1;
            }
        }
        Data::new(element_grid_dir)
    }
}
