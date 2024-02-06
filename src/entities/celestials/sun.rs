use bevy::log::info;

use crate::{
    entities::celestials::celestial::CelestialData,
    physics::{
        self,
        fallingsand::{
            data::element_directory::ElementGridDir, elements::element::ElementType,
            mesh::coordinate_directory::CoordinateDirBuilder, util::vectors::ChunkIjkVector,
        },
    },
};

pub struct SunBuilder {
    cell_radius: f32,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    first_num_radial_chunks: usize,
    max_radial_lines_per_chunk: usize,
    max_concentric_circles_per_chunk: usize,
}

impl Default for SunBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SunBuilder {
    pub fn new() -> Self {
        Self {
            cell_radius: 10.0,
            num_layers: 4,
            first_num_radial_lines: 12,
            second_num_concentric_circles: 3,
            first_num_radial_chunks: 3,
            max_radial_lines_per_chunk: 128,
            max_concentric_circles_per_chunk: 128,
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

    pub fn first_num_radial_chunks(mut self, first_num_radial_chunks: usize) -> Self {
        self.first_num_radial_chunks = first_num_radial_chunks;
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
            .cell_radius(physics::heat::components::Length(self.cell_radius))
            .num_layers(self.num_layers)
            .first_num_radial_lines(self.first_num_radial_lines)
            .second_num_concentric_circles(self.second_num_concentric_circles)
            .first_num_radial_chunks(self.first_num_radial_chunks)
            .max_radial_lines_per_chunk(self.max_radial_lines_per_chunk)
            .max_concentric_circles_per_chunk(self.max_concentric_circles_per_chunk)
            .build();
        let mut element_grid_dir = ElementGridDir::new_empty(coordinate_dir);
        info!("Num elements: {}", element_grid_dir.get_total_num_cells());

        // Iterate over each layer of the element grid and fill it with the appropriate element
        for layer_num in 0..element_grid_dir.get_coordinate_dir().get_num_layers() {
            for j in 0..element_grid_dir
                .get_coordinate_dir()
                .get_layer_num_concentric_chunks(layer_num)
            {
                for k in 0..element_grid_dir
                    .get_coordinate_dir()
                    .get_layer_num_radial_chunks(layer_num)
                {
                    let chunk_idx = ChunkIjkVector::new(layer_num, j, k);
                    let element_grid = element_grid_dir.get_chunk_by_chunk_ijk_mut(chunk_idx);
                    element_grid.fill(ElementType::SolarPlasma);
                }
            }
        }
        CelestialData::new(element_grid_dir)
    }
}
