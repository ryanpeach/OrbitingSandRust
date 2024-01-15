use criterion::{black_box, criterion_group, Criterion};
use orbiting_sand::physics::fallingsand::data::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::fallingsand::util::enums::MeshDrawMode;
use orbiting_sand::physics::fallingsand::util::mesh::OwnedMeshData;

/// The default element grid directory for testing
fn get_element_grid_dir() -> ElementGridDir {
    let coordinate_dir = CoordinateDirBuilder::new()
        .cell_radius(1.0)
        .num_layers(8)
        .first_num_radial_lines(12)
        .second_num_concentric_circles(3)
        .first_num_radial_chunks(3)
        .max_concentric_circles_per_chunk(128)
        .max_radial_lines_per_chunk(128)
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

fn stitch_mesh(c: &mut Criterion) {
    let element_grid_dir = get_element_grid_dir();
    let coordinate_dir = element_grid_dir.get_coordinate_dir();
    let mesh = coordinate_dir.get_mesh_data(MeshDrawMode::TexturedMesh);
    let mesh = OwnedMeshData::combine(&mesh);

    c.bench_function("stitch_mesh", |b| {
        b.iter(|| {
            let _ = mesh.stitch_mesh();
        })
    });
}

fn deduplicate_vertexes(c: &mut Criterion) {
    let element_grid_dir = get_element_grid_dir();
    let coordinate_dir = element_grid_dir.get_coordinate_dir();
    let mesh = coordinate_dir.get_mesh_data(MeshDrawMode::TexturedMesh);
    let mesh = OwnedMeshData::combine(&mesh).stitch_mesh();

    c.bench_function("deduplicate_vertexes", |b| {
        b.iter(|| {
            let _ = mesh.deduplicate_vertexes();
        })
    });
}

criterion_group!(
    benches,
    bench_combine_meshes,
    stitch_mesh,
    deduplicate_vertexes
);
