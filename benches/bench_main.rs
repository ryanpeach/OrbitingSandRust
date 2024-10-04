use iai_callgrind::main;

pub mod physics;

use crate::physics::fallingsand::data::element_directory::element_directory_group;
use crate::physics::fallingsand::mesh::coordinate_dir::coordinate_dir_group;

main!(
    library_benchmark_groups = coordinate_dir_group,
    element_directory_group
);
