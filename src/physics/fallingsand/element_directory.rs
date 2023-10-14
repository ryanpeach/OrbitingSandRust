use std::collections::HashSet;

use uom::si::f64::Time;

use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_convolution::{
    ElementGridConvolutionNeighbors, ElementGridConvolutionNeighborsChunkIdx,
};
use super::element_grid::ElementGrid;
use super::util::functions::modulo;
use super::util::grid::Grid;
use super::util::image::RawImage;
use super::util::vectors::{ChunkIjkVector, JkVector};

use itertools::Itertools;
use rayon::prelude::*;

/* Main Struct */
/// An element grid directory is like a coordinate directory, but for element grids
/// It follow the same layer structure
/// There is a coordinate directory at the root, but also each ElementGrid has its own
/// copy of the chunk coordinates associated with it for convenience
pub struct ElementGridDir {
    coords: CoordinateDir,
    chunks: Vec<Grid<Option<ElementGrid>>>,
    process_count: usize,
}

impl ElementGridDir {
    pub fn new_empty(coords: CoordinateDir) -> Self {
        let mut chunks: Vec<Grid<Option<ElementGrid>>> =
            Vec::with_capacity(coords.get_num_layers());
        for i in 0..coords.get_num_chunks() {
            let j_size = coords.get_chunk_layer_num_concentric_circles(i);
            let k_size = coords.get_chunk_layer_num_radial_lines(i);
            let mut layer = Grid::new_empty(k_size, j_size);
            for j in 0..j_size {
                for k in 0..k_size {
                    let element_grid =
                        ElementGrid::new_empty(coords.get_chunk_at_idx(ChunkIjkVector { i, j, k }));
                    layer.replace(JkVector { j, k }, Some(element_grid));
                }
            }
            chunks.push(layer);
        }
        Self {
            coords,
            chunks,
            process_count: 0,
        }
    }

    // TODO: This needs testing
    fn get_next_targets(&self) -> HashSet<ChunkIjkVector> {
        // let mut out = HashSet::new();
        // for j in (0..self
        //     .coords
        //     .get_total_number_chunks_in_concentric_circle_dimension())
        //     .skip((self.process_count / 3) % 3)
        //     .step_by(3)
        // {
        //     let (layer_num, chunk_layer_concentric_circle) = self
        //         .coords
        //         .get_layer_num_from_absolute_chunk_concentric_circle(j);
        //     for k in (0..self.coords.get_chunk_layer_num_radial_lines(layer_num))
        //         .skip(self.process_count % 3)
        //         .step_by(3)
        //     {
        //         out.insert(ChunkIjkVector {
        //             i: layer_num,
        //             j: chunk_layer_concentric_circle,
        //             k,
        //         });
        //     }
        // }
        // out
        unimplemented!()
    }

    // TODO: This needs testing
    fn get_chunk_top_neighbors(
        &self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighborsChunkIdx {
        let top_chunk_in_layer = self.coords.get_chunk_layer_num_concentric_circles(coord.i);
        let top_layer = self.coords.get_num_layers() - 1;
        let radial_lines = |i: usize| self.coords.get_chunk_layer_num_radial_lines(i);
        let k_isize = coord.k as isize;
        let mut out: ElementGridConvolutionNeighborsChunkIdx =
            ElementGridConvolutionNeighborsChunkIdx::new();

        let mut make_vector = |i: usize, j: usize, k: isize| {
            let this = ChunkIjkVector {
                i,
                j,
                k: modulo(k, radial_lines(i) as isize) as usize,
            };
            out.insert(this)
        };

        // Default neighbors (middle of stuff)
        let mut default_neighbors = || {
            make_vector(coord.i, coord.j + 1, k_isize - 1);
            make_vector(coord.i, coord.j + 1, k_isize);
            make_vector(coord.i, coord.j + 1, k_isize + 1);
            make_vector(coord.i, coord.j + 1, k_isize + 2);
        };

        match (coord.i, coord.j) {
            (i, _) if i == top_layer => match coord.j {
                j if j == top_chunk_in_layer => {}
                _ => {
                    default_neighbors();
                }
            },
            (_, j) if j == top_chunk_in_layer => {
                make_vector(coord.i + 1, 0, k_isize * 2 - 1);
                make_vector(coord.i + 1, 0, k_isize * 2);
                make_vector(coord.i + 1, 0, k_isize * 2 + 1);
                make_vector(coord.i + 1, 0, k_isize * 2 + 2);
            }
            _ => {
                default_neighbors();
            }
        }
        out
    }

    // TODO: This needs testing
    fn get_chunk_left_right_neighbors(
        &self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighborsChunkIdx {
        let mut out: ElementGridConvolutionNeighborsChunkIdx =
            ElementGridConvolutionNeighborsChunkIdx::new();
        let left = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(
                coord.k as isize - 1,
                self.coords.get_chunk_layer_num_radial_lines(coord.i) as isize,
            ) as usize,
        };
        out.insert(left);
        let right = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(
                coord.k as isize + 1,
                self.coords.get_chunk_layer_num_radial_lines(coord.i) as isize,
            ) as usize,
        };
        out.insert(right);
        out
    }

    // TODO: This needs testing
    fn get_chunk_bottom_neighbors(
        &self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighborsChunkIdx {
        let bottom_chunk_in_layer = 0usize;
        let bottom_layer = 0usize;
        let radial_lines = |i: usize| self.coords.get_chunk_layer_num_radial_lines(i);
        let top_chunk_in_prev_layer =
            |i: usize| self.coords.get_chunk_layer_num_concentric_circles(i - 1) - 1;
        let k_isize = coord.k as isize;
        let mut out: ElementGridConvolutionNeighborsChunkIdx =
            ElementGridConvolutionNeighborsChunkIdx::new();

        let mut make_vector = |i: usize, j: usize, k: isize| {
            let this = ChunkIjkVector {
                i,
                j,
                k: modulo(k, radial_lines(i) as isize) as usize,
            };
            out.insert(this)
        };

        // Default neighbors
        let mut default_neighbors = || {
            make_vector(coord.i, coord.j - 1, k_isize + 1);
            make_vector(coord.i, coord.j - 1, k_isize);
            make_vector(coord.i, coord.j - 1, k_isize - 1);
        };

        match (coord.i, coord.j) {
            (i, j) if i == bottom_layer && j == bottom_chunk_in_layer => {}
            // If going down a layer but you are not at the bottom
            (i, j) if j == bottom_chunk_in_layer => {
                make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2 + 1);
                make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2);
                // This is not -1 because integer division naturally rounds down
            }
            _ => default_neighbors(),
        }
        out
    }

    fn get_chunk_neighbors(
        &self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighborsChunkIdx {
        let top = self.get_chunk_top_neighbors(coord);
        let lr = self.get_chunk_left_right_neighbors(coord);
        let bottom = self.get_chunk_bottom_neighbors(coord);
        let mut out = ElementGridConvolutionNeighborsChunkIdx::new();
        out.extend(top);
        out.extend(lr);
        out.extend(bottom);
        out
    }

    fn package_this_convolution(
        &mut self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighbors {
        println!("Packaging convolution for chunk {:?}", coord);
        let neighbors = self.get_chunk_neighbors(coord);
        let mut out = ElementGridConvolutionNeighbors::new();
        for neighbor in neighbors {
            let chunk = self.chunks[neighbor.i]
                .replace(neighbor.to_jk_vector(), None)
                .unwrap();
            out.insert(neighbor, chunk);
        }
        out
    }

    // This takes ownership of the chunk and all its neighbors from the directory
    // and puts them into a target vector and a vector of convolutions
    // The taget vector and convolution vectors will then be iterated on in parallel
    fn package_convolutions(&mut self) -> (Vec<ElementGridConvolutionNeighbors>, Vec<ElementGrid>) {
        let target_chunk_coords = self.get_next_targets();

        let mut convolutions = Vec::new();
        let mut target_chunks = Vec::new();

        for coord in &target_chunk_coords {
            convolutions.push(self.package_this_convolution(*coord));

            let chunk = self.chunks[coord.i]
                .replace(coord.to_jk_vector(), None)
                .unwrap();
            target_chunks.push(chunk);
        }

        (convolutions, target_chunks)
    }
    /// TODO: This needs testing
    /// The reverse of the package_convolutions function
    /// Puts the chunks back into the directory exactly where they were taken from
    /// This is easy because all elementgrids contain a coordinate
    fn unpackage_convolutions(
        &mut self,
        convolutions: Vec<ElementGridConvolutionNeighbors>,
        target_chunks: Vec<ElementGrid>,
    ) {
        for (mut target_chunk, this_conv) in target_chunks.into_iter().zip(convolutions.into_iter())
        {
            target_chunk.set_already_processed(true);
            let coord = target_chunk.get_chunk_coords();
            println!(
                "Unpackaging convolution for chunk {:?}",
                coord.get_chunk_idx()
            );
            let prev = self.chunks[coord.get_layer_num()].replace(
                JkVector {
                    j: coord.get_start_concentric_circle_layer_relative(),
                    k: coord.get_start_radial_line(),
                },
                Some(target_chunk),
            );
            debug_assert!(prev.is_none(), "Somehow this chunk was already replaced.");
            for (neighbor_idx, neighbor) in this_conv.into_iter() {
                let prev = self.chunks[neighbor_idx.i]
                    .replace(neighbor_idx.to_jk_vector(), Some(neighbor));
                debug_assert!(prev.is_none(), "Somehow this chunk was already replaced.");
            }
        }
    }

    fn get_unprocessed_chunk_idxs(&self) -> Vec<ChunkIjkVector> {
        let mut out = Vec::new();
        for i in 0..self.coords.get_num_chunks() {
            let j_size = self.coords.get_layer_num_concentric_circles(i);
            let k_size = self.coords.get_layer_num_radial_lines(i);
            for j in 0..j_size {
                for k in 0..k_size {
                    let coord = ChunkIjkVector { i, j, k };
                    if !self.get_chunk_by_chunk_ijk(coord).get_already_processed() {
                        out.push(coord);
                    }
                }
            }
        }
        out
    }

    fn unlock_all_chunks(&mut self) {
        for i in 0..self.coords.get_num_chunks() {
            let j_size = self.coords.get_layer_num_concentric_circles(i);
            let k_size = self.coords.get_layer_num_radial_lines(i);
            for j in 0..j_size {
                for k in 0..k_size {
                    let coord = ChunkIjkVector { i, j, k };
                    self.get_chunk_by_chunk_ijk_mut(coord)
                        .set_already_processed(false);
                }
            }
        }
    }

    /// Do one iteration of processing on the grid
    /// There are four passes in total, each call does one pass
    /// The passes ensure that no two adjacent elementgrids are processed at the same time
    /// This is important because elementgrids can effect one another at a maximum range of
    /// the size of one elementgrid.
    pub fn process(&mut self, delta: Time) {
        // The main parallel logic
        let (mut convolutions, mut target_chunks) = self.package_convolutions();
        convolutions
            .par_iter_mut()
            .zip(target_chunks.par_iter_mut())
            .for_each(|(convolution, target_chunk)| {
                target_chunk.process(convolution, delta);
            });
        self.unpackage_convolutions(convolutions, target_chunks);

        // Increment the process count and check for errors
        self.process_count += 1;
        if self.process_count % 9 == 0 {
            let unprocessed = self.get_unprocessed_chunk_idxs();
            debug_assert_ne!(
                unprocessed.len(),
                0,
                "After 9 iterations not all chunks are processed. Missing {:?}",
                unprocessed
            );
            self.unlock_all_chunks();
        }
    }

    /// Get the number of chunks from the coordinate directory
    pub fn get_num_chunks(&self) -> usize {
        self.coords.get_num_chunks()
    }

    /// Gets the chunk at the given index
    /// Errors if it is currently borrowed
    pub fn get_chunk_by_chunk_ijk(&self, coord: ChunkIjkVector) -> &ElementGrid {
        self.chunks[coord.i]
            .get(coord.to_jk_vector())
            .as_ref()
            .unwrap()
    }
    /// Gets the chunk at the given index mutably
    /// Errors if it is currently borrowed
    pub fn get_chunk_by_chunk_ijk_mut(&mut self, coord: ChunkIjkVector) -> &mut ElementGrid {
        self.chunks[coord.i]
            .get_mut(coord.to_jk_vector())
            .as_mut()
            .unwrap()
    }

    pub fn get_coordinate_dir(&self) -> &CoordinateDir {
        &self.coords
    }
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    pub fn get_textures(&self) -> Vec<RawImage> {
        (0..self.coords.get_num_layers())
            .flat_map(|i| {
                let j_range = 0..self.coords.get_chunk_layer_num_concentric_circles(i);
                let k_range = 0..self.coords.get_chunk_layer_num_radial_lines(i);
                j_range
                    .cartesian_product(k_range)
                    .map(move |(j, k)| (i, j, k))
            })
            .par_bridge() // Convert to parallel iterator
            .map(|(i, j, k)| {
                let coord = ChunkIjkVector { i, j, k };
                self.get_chunk_by_chunk_ijk(coord).get_texture()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::physics::fallingsand::coordinates::coordinate_directory::CoordinateDirBuilder;

    use super::*;

    /// The default element grid directory for testing
    fn get_element_grid_dir() -> ElementGridDir {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_cells(64 * 64)
            .build();
        ElementGridDir::new_empty(coordinate_dir)
    }

    mod neighbors {

        use super::*;

        /// Going to verify the chunk grid sizes before we start testing, and so we can know if they change
        #[test]
        fn test_grid_sizes() {
            let element_grid_dir = get_element_grid_dir();
            assert_eq!(element_grid_dir.len(), 7);

            // Core
            assert_eq!(element_grid_dir.chunks[0].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[0].get_width(), 1);

            // Layer 1
            assert_eq!(element_grid_dir.chunks[1].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[1].get_width(), 1);

            // Layer 2
            assert_eq!(element_grid_dir.chunks[2].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[2].get_width(), 1);

            // Layer 3
            assert_eq!(element_grid_dir.chunks[3].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[3].get_width(), 1);

            // Layer 4
            assert_eq!(element_grid_dir.chunks[4].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[4].get_width(), 1);

            // Layer 5
            assert_eq!(element_grid_dir.chunks[5].get_height(), 1);
            assert_eq!(element_grid_dir.chunks[5].get_width(), 6);

            // Layer 6
            assert_eq!(element_grid_dir.chunks[6].get_height(), 3);
            assert_eq!(element_grid_dir.chunks[6].get_width(), 12);

            // Layer 7
            assert_eq!(element_grid_dir.chunks[7].get_height(), 6);
            assert_eq!(element_grid_dir.chunks[7].get_width(), 24);
        }

        #[test]
        fn test_get_chunk_neighbors() {
            let element_grid_dir = get_element_grid_dir();

            // Core
            {
                let coord = ChunkIjkVector { i: 0, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 1);
                assert!(neighbors.contains(&ChunkIjkVector { i: 1, j: 0, k: 0 }));
            }

            // Layer 1
            {
                let coord = ChunkIjkVector { i: 1, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 2);
                assert!(neighbors.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
            }

            // Layer 2
            {
                let coord = ChunkIjkVector { i: 2, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 2);
                assert!(neighbors.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 1, j: 0, k: 0 }));
            }

            // Layer 3
            {
                let coord = ChunkIjkVector { i: 3, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 2);
                assert!(neighbors.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));
            }

            // Layer 4
            // This is the first special one, because it is the first layer with more than one chunk
            // It will have 6 chunks above it and one below it
            {
                let coord = ChunkIjkVector { i: 4, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 7);
                assert!(neighbors.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 1 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 2 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 3 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 4 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 5 }));
            }

            // Layer 5,
            // This one splits radially, so it will have left and right neighbors
            // The neighbor below it still just has one chunk, so one bottom neighbor
            // The neighbor above it will have a tl, t, and tr
            {
                let coord = ChunkIjkVector { i: 5, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.len(), 7);
                assert!(neighbors.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 1 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 3 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 1 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 2 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 11 }));
            }

            // Layer 6
            // Now we have both up and down neighbors with multiple chunks
            // But we only have down when j == 0
            // And we only have up when j == 2
            // Deal with all 3 layers because we havent seen a middle case yet
            unimplemented!();

            // Layer 7
            // Now we just need to implement the top case, where there is nothing left above us
            unimplemented!();
        }
    }

    // mod get_next_targets {
    //     use super::*;

    //     /// Test that every chunk is targetted exactly once in 9 iterations
    //     fn test_get_next_targets_full_coverage() {
    //         let element_grid_dir = get_element_grid_dir();
    //         let all_targets = HashSet::new();
    //         for _ in range(9) {
    //             let targets = element_grid_dir.get_next_targets();
    //             all_targets.extend(targets);
    //         }
    //         assert_eq!(all_targets.len(), element_grid.get_num_chunks());
    //     }

    //     /// Test that no chunk is targetted twice in 9 iterations
    //     fn test_get_next_targets_no_duplicates() {
    //         let element_grid_dir = get_element_grid_dir();
    //         let all_targets = HashSet::new();
    //         for _ in range(9) {
    //             let targets = element_grid_dir.get_next_targets();
    //             for t in targets {
    //                 assert!(!all_targets.contains(t));
    //             }
    //         }
    //     }

    //     /// Make some manual assertions
    //     fn test_get_next_targets_manual() {
    //         let element_grid_dir = get_element_grid_dir();
    //         let all_targets = HashSet::new();

    //         {
    //             let targets = element_grid_dir.get_next_targets();
    //             all_targets.extend(targets);

    //             // Core
    //             assert!(targets.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));

    //             // Layer 3
    //             assert!(targets.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));

    //             // Layer 6
    //             assert!(targets.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
    //         }

    //         {
    //             let targets = element_grid_dir.get_next_targets();
    //             all_targets.extend(targets);

    //             // Its important that if we exhaust a layer we stop targetting it
    //             assert!(!targets.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
    //         }

    //         unimplemented!()
    //     }
    // }
}
