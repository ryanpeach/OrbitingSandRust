use std::time::Duration;

use bevy::core::FrameCount;
use bevy::time::Time;
use iai_callgrind::{library_benchmark, library_benchmark_group, LibraryBenchmarkConfig};
use lazy_static::lazy_static;
use orbiting_sand::physics::fallingsand::mesh::coordinate_dir::CoordinateDir;
use orbiting_sand::physics::fallingsand::{
    data::element_directory::ElementGridDir, mesh::coordinate_dir::Builder,
};
use orbiting_sand::physics::orbits::components::Length;
use orbiting_sand::physics::util::clock::Clock;

lazy_static! {
    static ref COORDINATE_DIR: CoordinateDir = Builder::new()
        .cell_width(Length(1.0))
        .num_layers(11)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(3)
        .max_concentric_circles_per_chunk(64)
        .max_radial_lines_per_chunk(64)
        .build();
}

/// All the element grid directories that need processing
mod get_element_grid_dir {
    use super::*;
    /// The default element grid directory for testing
    pub fn empty() -> ElementGridDir {
        ElementGridDir::new_empty(COORDINATE_DIR.clone())
    }
}

// This one really does not change depending on the type of element grid directory
#[library_benchmark(config = LibraryBenchmarkConfig::default().valgrind_args(["--num-callers=10"]))]
fn bench_get_textures() {
    let element_grid_dir = get_element_grid_dir::empty();
    element_grid_dir.textures();
}

// Fully processes a set of element grid directories as defined in the get_element_grid_dir module
#[library_benchmark(config = LibraryBenchmarkConfig::default().valgrind_args(["--num-callers=10"]))]
#[benches::multiple(
    (get_element_grid_dir::empty(), 1)
)]
fn bench_process(mut element_grid_dir: ElementGridDir, nb_iterations: usize) {
    let mut current_time = Clock::new(Time::default(), FrameCount(0));
    for _ in 0..nb_iterations {
        element_grid_dir.process_full(current_time);
        current_time.update(Duration::from_secs(1));
    }
}

library_benchmark_group!(
  name = element_directory_group;
  benchmarks=
    bench_get_textures,
    bench_process,
);
