use std::collections::{HashMap, HashSet};

use uom::si::f64::Time;

use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_convolution::{
    BottomNeighbors, ElementGridConvolutionNeighbors, ElementGridConvolutionNeighborsChunkIdx,
    LeftRightNeighbors, TopNeighbors,
};
use super::element_grid::ElementGrid;
use super::util::functions::modulo;
use super::util::grid::Grid;
use super::util::image::RawImage;
use super::util::vectors::{ChunkIjkVector, JkVector};

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
        for i in 0..coords.get_num_layers() {
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
    fn get_next_targets(&mut self) -> HashSet<ChunkIjkVector> {
        let mut out = HashSet::new();

        // We are going to iterate up every j chunk ignoring the layer they are on, so we need the total number of them
        let j_size = self
            .coords
            .get_total_number_chunks_in_concentric_circle_dimension();
        debug_assert!(
            j_size % 3 == 0,
            "Number of chunks in concentric circle dimension must be divisible by 3, but it is {}",
            j_size
        );

        // We are going to iterate 9 times in self.process_count
        // We will start one forward every 3 iterations in the j dim
        // We will start one forward every iteration in the k dim, looping every 3 iterations
        let start_j = (self.process_count / 3) % 3;
        let start_k = self.process_count % 3;

        // We need to step by 3 to prevent overlap. Think of a 3x3 convolution
        for j in (start_j..j_size).step_by(3) {
            // Get our layer shape
            let (layer_num, chunk_layer_concentric_circle) = self
                .coords
                .get_layer_num_from_absolute_chunk_concentric_circle(j);
            let chunk_layer_radial_lines = self.coords.get_chunk_layer_num_radial_lines(layer_num);
            debug_assert!(
                chunk_layer_radial_lines == 1 || chunk_layer_radial_lines % 3 == 0,
                "Chunk layer radial lines must be divisible by 3, but it is {}",
                chunk_layer_radial_lines
            );

            // Some layers just have one chunk, we need to only produce these values on the first iteration of the self.process_count cycle
            if chunk_layer_radial_lines == 1 && ((self.process_count / 3) % 3) != 0 {
                continue;
            }

            // We need to step by 3 to prevent overlap. Think of a 3x3 convolution
            for k in (start_k..chunk_layer_radial_lines).step_by(3) {
                out.insert(ChunkIjkVector {
                    i: layer_num,
                    j: chunk_layer_concentric_circle,
                    k,
                });
            }
        }
        self.process_count += 1;
        out
    }

    // TODO: This needs testing
    fn get_chunk_top_neighbors(&self, coord: ChunkIjkVector) -> TopNeighbors {
        let top_chunk_in_layer = self.coords.get_chunk_layer_num_concentric_circles(coord.i) - 1;
        let top_layer = self.coords.get_num_layers() - 1;
        let radial_lines = |i: usize| self.coords.get_chunk_layer_num_radial_lines(i);
        let k_isize = coord.k as isize;

        // A convenience function for making a vector and adding it to the out set
        let make_vector = |i: usize, j: usize, k: isize| -> ChunkIjkVector {
            ChunkIjkVector {
                i,
                j,
                k: modulo(k, radial_lines(i) as isize) as usize,
            }
        };

        // There is a special case where you go from a single chunk layer to a multi chunk layer, where
        // all of the next chunks layers are above you
        if coord.i != top_layer && radial_lines(coord.i) == 1 && radial_lines(coord.i + 1) != 1 {
            let next_layer = coord.i + 1;
            let next_layer_radial_lines = radial_lines(next_layer);
            let mut out = Vec::new();
            for k in 0..next_layer_radial_lines {
                let next_layer_chunk = ChunkIjkVector {
                    i: next_layer,
                    j: 0,
                    k,
                };
                out.push(next_layer_chunk);
            }
            return TopNeighbors::MultiChunkLayerAbove { chunks: out };
        }

        // Default neighbors (assuming you are in the middle of stuff, not on a layer boundary)
        let default_neighbors = TopNeighbors::Normal {
            tl: make_vector(coord.i, coord.j + 1, k_isize + 1),
            t: make_vector(coord.i, coord.j + 1, k_isize),
            tr: make_vector(coord.i, coord.j + 1, k_isize - 1),
        };

        match (coord.i, coord.j) {
            (i, _) if i == top_layer => match coord.j {
                j if j == top_chunk_in_layer => TopNeighbors::TopOfGrid,
                _ => default_neighbors,
            },
            (i, j) if j == top_chunk_in_layer && radial_lines(i) != radial_lines(i + 1) => {
                TopNeighbors::LayerTransition {
                    tl: make_vector(i + 1, 0, k_isize * 2 + 2),
                    t1: make_vector(i + 1, 0, k_isize * 2 + 1),
                    t0: make_vector(i + 1, 0, k_isize * 2),
                    tr: make_vector(i + 1, 0, k_isize * 2 - 1),
                }
            }
            (i, j) if j == top_chunk_in_layer && radial_lines(i + 1) == 1 => {
                TopNeighbors::SingleChunkLayerAbove {
                    t: make_vector(i + 1, 0, 0),
                }
            }
            (i, j) if j == top_chunk_in_layer && radial_lines(i) == radial_lines(i + 1) => {
                TopNeighbors::Normal {
                    tl: make_vector(i + 1, 0, k_isize + 1),
                    t: make_vector(i + 1, 0, k_isize),
                    tr: make_vector(i + 1, 0, k_isize - 1),
                }
            }
            _ => default_neighbors,
        }
    }

    // TODO: This needs testing
    fn get_chunk_left_right_neighbors(&self, coord: ChunkIjkVector) -> LeftRightNeighbors {
        let num_radial_chunks = self.coords.get_chunk_layer_num_radial_lines(coord.i);
        if num_radial_chunks == 1 {
            return LeftRightNeighbors::SingleChunkLayer;
        }
        let left = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(coord.k as isize - 1, num_radial_chunks as isize) as usize,
        };
        debug_assert_ne!(left, coord);
        let right = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(coord.k as isize + 1, num_radial_chunks as isize) as usize,
        };
        debug_assert_ne!(right, coord);
        debug_assert_ne!(left, right);
        LeftRightNeighbors::LR { l: left, r: right }
    }

    // TODO: This needs testing
    fn get_chunk_bottom_neighbors(&self, coord: ChunkIjkVector) -> BottomNeighbors {
        let bottom_chunk_in_layer = 0usize;
        let bottom_layer = 0usize;
        let radial_lines = |i: usize| self.coords.get_chunk_layer_num_radial_lines(i);
        let top_chunk_in_prev_layer =
            |i: usize| self.coords.get_chunk_layer_num_concentric_circles(i - 1) - 1;
        let k_isize = coord.k as isize;

        let make_vector = |i: usize, j: usize, k: isize| -> ChunkIjkVector {
            ChunkIjkVector {
                i,
                j,
                k: modulo(k, radial_lines(i) as isize) as usize,
            }
        };

        match (coord.i, coord.j, coord.k) {
            (i, j, _) if i == bottom_layer && j == bottom_chunk_in_layer => {
                BottomNeighbors::BottomOfGrid
            }
            (i, j, _) if j == bottom_chunk_in_layer && radial_lines(i - 1) == 1 => {
                BottomNeighbors::FullLayerBelow {
                    b: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), 0),
                }
            }
            // If going down a layer but you are not at the bottom
            (i, j, k)
                if j == bottom_chunk_in_layer
                    && radial_lines(i) != radial_lines(i - 1)
                    && k % 2 == 0 =>
            {
                BottomNeighbors::LayerTransition {
                    bl: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2),
                    br: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2 - 1),
                }
            }
            (i, j, k)
                if j == bottom_chunk_in_layer
                    && radial_lines(i) != radial_lines(i - 1)
                    && k % 2 == 1 =>
            {
                BottomNeighbors::LayerTransition {
                    bl: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2 + 1),
                    br: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize / 2),
                }
            }
            (i, j, _) if j == bottom_chunk_in_layer && radial_lines(i) == radial_lines(i - 1) => {
                BottomNeighbors::Normal {
                    bl: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize + 1),
                    b: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize),
                    br: make_vector(coord.i - 1, top_chunk_in_prev_layer(i), k_isize - 1),
                }
            }
            _ => BottomNeighbors::Normal {
                bl: make_vector(coord.i, coord.j - 1, k_isize + 1),
                b: make_vector(coord.i, coord.j - 1, k_isize),
                br: make_vector(coord.i, coord.j - 1, k_isize - 1),
            },
        }
    }

    fn get_chunk_neighbors(
        &self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighborsChunkIdx {
        let top = self.get_chunk_top_neighbors(coord);
        let left_right = self.get_chunk_left_right_neighbors(coord);
        let bottom = self.get_chunk_bottom_neighbors(coord);
        ElementGridConvolutionNeighborsChunkIdx {
            top,
            left_right,
            bottom,
        }
    }

    fn package_this_convolution(
        &mut self,
        coord: ChunkIjkVector,
    ) -> ElementGridConvolutionNeighbors {
        println!("Packaging convolution for chunk {:?}", coord);
        let neighbors = self.get_chunk_neighbors(coord);
        let mut out = HashMap::new();
        for neighbor in neighbors.iter() {
            let chunk = self.chunks[neighbor.i]
                .replace(neighbor.to_jk_vector(), None)
                .unwrap();
            out.insert(neighbor, chunk);
        }
        ElementGridConvolutionNeighbors::new(neighbors, out)
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
        for i in 0..self.coords.get_num_layers() {
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
        for i in 0..self.coords.get_num_layers() {
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
    pub fn get_textures(&self) -> Vec<Grid<RawImage>> {
        let mut out = Vec::new();
        for i in 0..self.coords.get_num_layers() {
            let j_size = self.coords.get_chunk_layer_num_concentric_circles(i);
            let k_size = self.coords.get_chunk_layer_num_radial_lines(i);
            let mut layer = Grid::new_empty(k_size, j_size);
            for j in 0..j_size {
                for k in 0..k_size {
                    let coord = ChunkIjkVector { i, j, k };
                    layer.replace(
                        JkVector { j, k },
                        self.get_chunk_by_chunk_ijk(coord).get_texture(),
                    );
                }
            }
            out.push(layer);
        }
        out
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
            .num_layers(9)
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
            assert_eq!(element_grid_dir.len(), 9);

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
            assert_eq!(element_grid_dir.chunks[6].get_width(), 6);

            // Layer 7
            assert_eq!(element_grid_dir.chunks[7].get_height(), 6);
            assert_eq!(element_grid_dir.chunks[7].get_width(), 12);

            // Layer 8
            assert_eq!(element_grid_dir.chunks[8].get_height(), 12);
            assert_eq!(element_grid_dir.chunks[8].get_width(), 24);
        }

        #[test]
        fn test_get_chunk_neighbors() {
            let element_grid_dir = get_element_grid_dir();

            // Core
            {
                let coord = ChunkIjkVector { i: 0, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 1, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 1, j: 0, k: 0 }));
            }

            // Layer 1
            {
                let coord = ChunkIjkVector { i: 1, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 2, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
            }

            // Layer 2
            {
                let coord = ChunkIjkVector { i: 2, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 2, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 1, j: 0, k: 0 }));
            }

            // Layer 3
            {
                let coord = ChunkIjkVector { i: 3, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 2, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));
            }

            // Layer 4
            // This is the first special one, because it is the first layer with more than one chunk
            // It will have 6 chunks above it and one below it
            {
                let coord = ChunkIjkVector { i: 4, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 7, "{:?}", neighbors);
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
                assert_eq!(neighbors.iter().count(), 6, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 1 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 5 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 0 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 1 }));
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 5 }));
            }

            // Layer 6
            // This is the first layer that splits concentrically
            // It skips splitting radially just one time, so the layer below has the same
            {
                let coord = ChunkIjkVector { i: 6, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 8, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 0 })); // t
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 1 })); // tl
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 5 })); // tr
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 1 })); // l
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 5 })); // r
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 1 })); // bl
                assert!(neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 0 })); // b
                assert!(
                    neighbors.contains(&ChunkIjkVector { i: 5, j: 0, k: 5 }),
                    "{:?}",
                    neighbors
                ); // br
            }

            // Now go to a normal square
            {
                let coord = ChunkIjkVector { i: 6, j: 1, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 8, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 0 })); // t
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 1 })); // tl
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 5 })); // tr
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 1 })); // l
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 5 })); // r
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 1 })); // bl
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 0 })); // b
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 0, k: 5 }));
                // br
            }

            // Now go to the top of the layer
            // Because there is a doubling above this layer, then this has 2 top layers
            {
                let coord = ChunkIjkVector { i: 6, j: 2, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 9, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 2 })); // tl
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 1 })); // t0
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 0 })); // t1
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 11 })); // tr
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 1 })); // l
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 5 })); // r
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 1 })); // bl
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 0 })); // b
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 1, k: 5 }));
                // br
            }

            // Layer 7
            // This is the first layer that splits concentrically and radially
            {
                let coord = ChunkIjkVector { i: 7, j: 0, k: 0 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 7, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 0 })); // t
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 1 })); // tl
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 11 })); // tr
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 1 })); // l
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 11 })); // r
                assert!(
                    neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 0 }),
                    "{:?}",
                    neighbors
                ); // bl
                assert!(
                    neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 5 }),
                    "{:?}",
                    neighbors
                ); // br
            }

            // Go k+1 to test how going down left and right changes every other step
            // Because the one below you will "extend" more rightward or more leftward, so there
            // isnt really a bottom and whether its more leftward or rightward depends on which
            // side of it you are on
            {
                let coord = ChunkIjkVector { i: 7, j: 0, k: 1 };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert_eq!(neighbors.iter().count(), 7, "{:?}", neighbors);
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 1 })); // t
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 2 })); // tl
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 1, k: 0 })); // tr
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 2 })); // l
                assert!(neighbors.contains(&ChunkIjkVector { i: 7, j: 0, k: 0 })); // r
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 1 })); // bl
                assert!(neighbors.contains(&ChunkIjkVector { i: 6, j: 2, k: 0 }));
                // br
            }

            // Test the very last layer that it doesn't have anything above it
            {
                let coord = ChunkIjkVector {
                    i: element_grid_dir.get_coordinate_dir().get_num_layers() - 1,
                    j: 0,
                    k: 0,
                };
                let neighbors = element_grid_dir.get_chunk_neighbors(coord);
                assert!(!neighbors.contains(&ChunkIjkVector {
                    i: element_grid_dir.get_coordinate_dir().get_num_layers() - 1,
                    j: 0,
                    k: 0
                }));
            }
        }
    }

    mod get_next_targets {
        use super::*;

        /// Going to verify the chunk grid sizes before we start testing, and so we can know if they change
        #[test]
        fn test_grid_sizes() {
            let element_grid_dir = get_element_grid_dir();
            assert_eq!(element_grid_dir.len(), 9);

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
            assert_eq!(element_grid_dir.chunks[6].get_width(), 6);

            // Layer 7
            assert_eq!(element_grid_dir.chunks[7].get_height(), 6);
            assert_eq!(element_grid_dir.chunks[7].get_width(), 12);

            // Layer 8
            assert_eq!(element_grid_dir.chunks[8].get_height(), 12);
            assert_eq!(element_grid_dir.chunks[8].get_width(), 24);
        }

        /// Test that every chunk is targetted exactly once in 9 iterations
        #[test]
        fn test_get_next_targets_full_coverage() {
            let mut element_grid_dir = get_element_grid_dir();
            let mut all_targets = HashSet::new();
            for _ in 0..9 {
                let targets = element_grid_dir.get_next_targets();
                all_targets.extend(targets);
            }
            assert_eq!(
                all_targets.len(),
                element_grid_dir.coords.get_num_chunks(),
                "{:?}",
                all_targets
            );
        }

        /// Test that no chunk is targetted twice in 9 iterations
        #[test]
        fn test_get_next_targets_no_duplicates() {
            let mut element_grid_dir = get_element_grid_dir();
            let mut all_targets = HashSet::new();
            for process_count in 0..9 {
                let targets = element_grid_dir.get_next_targets();
                for t in &targets {
                    assert!(
                        !all_targets.contains(t),
                        "process_count: {}, t: {:?}",
                        process_count,
                        t
                    );
                }
                all_targets.extend(targets);
            }
        }

        #[test]
        #[warn(unused_variables)]
        fn test_get_next_targets_manual_j_dim() {
            let mut element_grid_dir = get_element_grid_dir();
            let all_targets_1 = element_grid_dir.get_next_targets();

            // For every j step by 3 we should
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 6, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 7, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 7, j: 3, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 8, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 8, j: 3, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 8, j: 6, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 8, j: 9, k: 0 }));

            let mut throw_away = element_grid_dir.get_next_targets();
            throw_away = element_grid_dir.get_next_targets();
            let all_targets_2 = element_grid_dir.get_next_targets();

            assert!(all_targets_2.contains(&ChunkIjkVector { i: 1, j: 0, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 4, j: 0, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 6, j: 1, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 7, j: 1, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 7, j: 4, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 1, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 4, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 7, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 10, k: 0 }));

            throw_away = element_grid_dir.get_next_targets();
            throw_away = element_grid_dir.get_next_targets();
            let all_targets_3 = element_grid_dir.get_next_targets();

            assert!(all_targets_2.contains(&ChunkIjkVector { i: 2, j: 0, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 5, j: 0, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 6, j: 2, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 7, j: 2, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 7, j: 5, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 2, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 5, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 8, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 8, j: 11, k: 0 }));
        }

        #[test]
        fn test_get_next_targets_manual_k_dim() {
            let mut element_grid_dir = get_element_grid_dir();
            let all_targets_1 = element_grid_dir.get_next_targets();

            // Same as first test
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 6, j: 0, k: 0 }));
            assert!(all_targets_1.contains(&ChunkIjkVector { i: 6, j: 0, k: 3 }));

            let all_targets_2 = element_grid_dir.get_next_targets();

            // Layers that only have one chunk should not repeat
            assert!(!all_targets_2.contains(&ChunkIjkVector { i: 0, j: 0, k: 0 }));
            assert!(!all_targets_2.contains(&ChunkIjkVector { i: 3, j: 0, k: 0 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 6, j: 0, k: 1 }));
            assert!(all_targets_2.contains(&ChunkIjkVector { i: 6, j: 0, k: 4 }));

            let all_targets_3 = element_grid_dir.get_next_targets();

            assert!(all_targets_3.contains(&ChunkIjkVector { i: 6, j: 0, k: 2 }));
            assert!(all_targets_3.contains(&ChunkIjkVector { i: 6, j: 0, k: 5 }));
        }
    }
}
