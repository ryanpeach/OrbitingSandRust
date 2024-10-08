use iai_callgrind::{
    main, EventKind, LibraryBenchmarkConfig,
    RegressionConfig,
};
pub mod physics;

use crate::physics::fallingsand::data::element_directory::element_directory_group;
use crate::physics::fallingsand::mesh::coordinate_dir::coordinate_dir_group;

main!(
    config = LibraryBenchmarkConfig::default()
        .regression(
            RegressionConfig::default()
                .limits([(EventKind::Ir, 5.0)])
        );
    library_benchmark_groups = coordinate_dir_group,
    element_directory_group
);
