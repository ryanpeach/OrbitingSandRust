use bevy::math::Vec2;
use iai_callgrind::{library_benchmark, library_benchmark_group};
use orbiting_sand::physics::{
    fallingsand::{
        mesh::coordinate_dir::{Builder, CoordinateDir},
        util::vectors::JkVector,
    },
    orbits::components::Length,
    util::vectors::RelXyPoint,
};

/// Iterate around the circle in every direction, targetting each cells midpoint, and return all
fn get_rel_pos_to_cell_idx_input_coords(coordinate_dir: &CoordinateDir) -> Vec<RelXyPoint> {
    let mut out = Vec::new();

    // Iterate around the core
    for k in 0..coordinate_dir
        .core_chunks()
        .get(JkVector::ZERO)
        .num_radial_lines()
    {
        // This radius and theta should define the midpoint of each cell
        let radius = coordinate_dir.cell_width().0 / 2.0;
        let theta = 2.0 * std::f32::consts::PI
            / coordinate_dir
                .core_chunks()
                .get(JkVector::ZERO)
                .num_radial_lines() as f32
            * (k as f32 + 0.5);
        let xycoord = RelXyPoint(Vec2 {
            x: radius * theta.cos(),
            y: radius * theta.sin(),
        });
        out.push(xycoord);
    }

    // Iterate around the rest
    for i in 1..coordinate_dir.num_layers() {
        let num_concentric_circles = coordinate_dir.layer_num_concentric_circles(i);
        let num_radial_lines = coordinate_dir.layer_num_radial_lines(i);
        for j in 0..num_concentric_circles {
            for k in 0..num_radial_lines {
                // This radius and theta should define the midpoint of each cell
                let radius = coordinate_dir.layer_start_radius(i)
                    + (coordinate_dir.layer_end_radius(i) - coordinate_dir.layer_start_radius(i))
                        / num_concentric_circles as f32
                        * (j as f32 + 0.5);
                let theta = 2.0 * std::f32::consts::PI / num_radial_lines as f32 * (k as f32 + 0.5);
                let xycoord = RelXyPoint(Vec2 {
                    x: radius * theta.cos(),
                    y: radius * theta.sin(),
                });
                out.push(xycoord);
            }
        }
    }
    out
}

fn setup_xycoords() -> Vec<RelXyPoint> {
    let coordinate_dir = Builder::new()
        .cell_width(Length(1.0))
        .num_layers(8)
        .first_num_radial_lines(8)
        .second_num_concentric_circles(2)
        .max_concentric_circles_per_chunk(64)
        .max_radial_lines_per_chunk(64)
        .build();

    get_rel_pos_to_cell_idx_input_coords(&coordinate_dir)
}

#[library_benchmark]
#[benches::multiple(setup_xycoords())]
fn bench_rel_pos_to_cell_idx(xycoord: RelXyPoint) {
    let _ = coordinate_dir
        .rel_pos_to_cell_idx(xycoord)
        .unwrap();
}

library_benchmark_group!(
  name=coordinate_dir_group;
  benchmarks =
    bench_rel_pos_to_cell_idx
);
