use criterion::{black_box, criterion_group, Criterion};
use orbiting_sand::physics::fallingsand::data::element_directory::ElementGridDir;
use orbiting_sand::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
use orbiting_sand::physics::fallingsand::util::enums::MeshDrawMode;
use orbiting_sand::physics::fallingsand::util::image::RawImage;
use orbiting_sand::physics::fallingsand::util::mesh::OwnedMeshData;

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

fn bench_combine_images(c: &mut Criterion) {
    let element_grid = get_element_grid_dir();
    let meshes = element_grid
        .get_coordinate_dir()
        .get_mesh_data(MeshDrawMode::TexturedMesh);
    let combined_meshes = OwnedMeshData::combine(&meshes);
    let textures = element_grid.get_textures();
    c.bench_function("combine_images", |b| {
        b.iter(|| {
            RawImage::combine(
                black_box(textures.clone()),
                black_box(combined_meshes.uv_bounds),
            );
        })
    });
}

criterion_group!(benches, bench_combine_images);
