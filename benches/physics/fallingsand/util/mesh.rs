use criterion::{black_box, criterion_group, Criterion};
use orbiting_sand::physics::fallingsand::coordinates::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::fallingsand::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::util::enums::MeshDrawMode;
use orbiting_sand::physics::fallingsand::util::mesh::OwnedMeshData;

/// The default element grid directory for testing
fn get_element_grid_dir() -> ElementGridDir {
    let coordinate_dir = CoordinateDirBuilder::new()
        .cell_radius(1.0)
        .num_layers(11)
        .first_num_radial_lines(6)
        .second_num_concentric_circles(3)
        .max_cells(64 * 64)
        .build();
    ElementGridDir::new_empty(coordinate_dir)
}

fn bench_combine_meshes(c: &mut Criterion) {
    let meshes = get_element_grid_dir()
        .get_coordinate_dir()
        .get_mesh_data(MeshDrawMode::TexturedMesh);
    c.bench_function("combine_meshes", |b| {
        b.iter(|| {
            OwnedMeshData::combine(black_box(&meshes[..]));
        })
    });
}

criterion_group!(benches, bench_combine_meshes);
