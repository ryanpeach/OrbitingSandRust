use iai_callgrind::{library_benchmark, library_benchmark_group};
use orbiting_sand::physics::fallingsand::{
    data::element_directory::ElementGridDir, mesh::coordinate_dir::Builder,
};
use orbiting_sand::physics::orbits::components::Length;

/// The default element grid directory for testing
fn get_element_grid_dir() -> ElementGridDir {
    let coordinate_dir = Builder::new()
        .cell_width(Length(1.0))
        .num_layers(11)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(3)
        .max_concentric_circles_per_chunk(64)
        .max_radial_lines_per_chunk(64)
        .build();
    ElementGridDir::new_empty(coordinate_dir)
}

#[library_benchmark]
fn bench_get_textures() {
    let element_grid_dir = get_element_grid_dir();
    element_grid_dir.textures();
}

library_benchmark_group!(
  name = element_directory_group;
  benchmarks=bench_get_textures
);
