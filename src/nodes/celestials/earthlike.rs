use crate::{
    physics::fallingsand::{
        data::element_directory::ElementGridDir, mesh::coordinate_directory::CoordinateDirBuilder,
    },
};

use super::celestial::Celestial;

pub struct EarthLikeBuilder {
    cell_radius: f32,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    first_num_radial_chunks: usize,
    max_radial_lines_per_chunk: usize,
    max_concentric_circles_per_chunk: usize,
}

impl EarthLikeBuilder {
    pub fn new() -> Self {
        Self {
            cell_radius: 1.0,
            num_layers: 7,
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

    pub fn build(&self) -> Celestial {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(self.cell_radius)
            .num_layers(self.num_layers)
            .first_num_radial_lines(self.first_num_radial_lines)
            .second_num_concentric_circles(self.second_num_concentric_circles)
            .first_num_radial_chunks(self.first_num_radial_chunks)
            .max_radial_lines_per_chunk(self.max_radial_lines_per_chunk)
            .max_concentric_circles_per_chunk(self.max_concentric_circles_per_chunk)
            .build();
        let element_grid_dir = ElementGridDir::new_empty(coordinate_dir);
        println!("Num elements: {}", element_grid_dir.get_total_num_cells());
        Celestial::new(element_grid_dir)
    }
}
