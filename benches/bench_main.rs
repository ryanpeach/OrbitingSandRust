use criterion::criterion_main;

mod physics;

criterion_main! {
    physics::fallingsand::coordinate_directory::benches,
    physics::fallingsand::util::image::benches,
    physics::fallingsand::element_directory::benches,
    physics::fallingsand::util::mesh::benches,
}
