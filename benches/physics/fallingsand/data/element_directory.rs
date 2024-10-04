use criterion::{criterion_group, Criterion};
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

fn bench_get_textures(c: &mut Criterion) {
    let element_grid_dir = get_element_grid_dir();
    c.bench_function("get_textures", |b| {
        b.iter(|| {
            element_grid_dir.textures();
        })
    });
}

criterion_group!(benches, bench_get_textures);
