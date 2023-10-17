use criterion::criterion_main;

mod physics;

criterion_main! {
    physics::fallingsand::coordinate_directory::benches,
}
