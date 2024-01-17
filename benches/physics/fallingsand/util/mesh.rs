
use orbiting_sand::physics::fallingsand::data::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;



/// The default element grid directory for testing
fn get_element_grid_dir() -> ElementGridDir {
    let coordinate_dir = CoordinateDirBuilder::new()
        .cell_radius(1.0)
        .num_layers(11)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(3)
        .max_concentric_circles_per_chunk(64)
        .max_radial_lines_per_chunk(64)
        .build();
    ElementGridDir::new_empty(coordinate_dir)
}

// criterion_group!(benches, bench_combine_meshes);
