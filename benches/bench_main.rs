use criterion::criterion_main;

mod physics;

criterion_main! {
    physics::fallingsand::mesh::coordinate_directory::benches,
    // physics::fallingsand::util::image::benches,
    physics::fallingsand::data::element_directory::benches,
    // physics::fallingsand::util::mesh::benches,
}
