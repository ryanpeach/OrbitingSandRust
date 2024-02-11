use bevy::log::info;

use crate::{
    entities::celestials::celestial::CelestialData,
    physics::{
        fallingsand::{
            data::element_directory::ElementGridDir, elements::element::ElementType,
            mesh::coordinate_directory::CoordinateDirBuilder, util::vectors::ChunkIjkVector,
        },
        heat::components::Length,
    },
};

pub struct EarthLikeBuilder {
    cell_radius: Length,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    first_num_tangential_chunkss: usize,
    max_radial_lines_per_chunk: usize,
    max_concentric_circles_per_chunk: usize,
}

impl Default for EarthLikeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EarthLikeBuilder {
    pub fn new() -> Self {
        Self {
            cell_radius: Length(1.0),
            num_layers: 8,
            first_num_radial_lines: 12,
            second_num_concentric_circles: 3,
            first_num_tangential_chunkss: 3,
            max_radial_lines_per_chunk: 128,
            max_concentric_circles_per_chunk: 128,
        }
    }

    pub fn cell_radius(mut self, cell_radius: Length) -> Self {
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

    pub fn first_num_tangential_chunkss(mut self, first_num_tangential_chunkss: usize) -> Self {
        self.first_num_tangential_chunkss = first_num_tangential_chunkss;
        self
    }

    pub fn max_radial_lines_per_chunk(mut self, max_radial_lines_per_chunk: usize) -> Self {
        self.max_radial_lines_per_chunk = max_radial_lines_per_chunk;
        self
    }

    pub fn max_concentric_circles_per_chunk(
        mut self,
        max_concentric_circles_per_chunk: usize,
    ) -> Self {
        self.max_concentric_circles_per_chunk = max_concentric_circles_per_chunk;
        self
    }

    pub fn build(&self) -> CelestialData {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(self.cell_radius)
            .num_layers(self.num_layers)
            .first_num_radial_lines(self.first_num_radial_lines)
            .second_num_concentric_circles(self.second_num_concentric_circles)
            .first_num_tangential_chunkss(self.first_num_tangential_chunkss)
            .max_radial_lines_per_chunk(self.max_radial_lines_per_chunk)
            .max_concentric_circles_per_chunk(self.max_concentric_circles_per_chunk)
            .build();
        let mut element_grid_dir = ElementGridDir::new_empty(coordinate_dir);
        info!("Num elements: {}", element_grid_dir.get_total_num_cells());

        // Iterate over each layer of the element grid and fill it with the appropriate element
        let mut total_j = 0;
        for layer_num in 0..element_grid_dir.get_coordinate_dir().get_num_layers() {
            for j in 0..element_grid_dir
                .get_coordinate_dir()
                .get_layer_num_concentric_chunks(layer_num)
            {
                for k in 0..element_grid_dir
                    .get_coordinate_dir()
                    .get_layer_num_tangential_chunkss(layer_num)
                {
                    let chunk_idx = ChunkIjkVector::new(layer_num, j, k);
                    let element_grid = element_grid_dir.get_chunk_by_chunk_ijk_mut(chunk_idx);
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
        CelestialData::new(element_grid_dir)
    }
}
