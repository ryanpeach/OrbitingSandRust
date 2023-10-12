use uom::si::f64::Time;

use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_convolution::{ElementGridConvolution, ElementGridConvolutionChunkIdx};
use super::element_grid::ElementGrid;
use super::util::functions::modulo;
use super::util::grid::Grid;
use super::util::image::RawImage;
use super::util::vectors::{ChunkIjkVector, JkVector};

use itertools::Itertools;
use rayon::prelude::*;

/* Return Types */
struct TopNeighbors {
    top_left: Option<ChunkIjkVector>,
    top_center_1: Option<ChunkIjkVector>,
    top_center_2: Option<ChunkIjkVector>, // Name it accordingly
    top_right: Option<ChunkIjkVector>,
}

struct LeftRightNeighbors {
    left: ChunkIjkVector,
    right: ChunkIjkVector,
}

struct BottomNeighbors {
    bottom_left: Option<ChunkIjkVector>,
    bottom_center: Option<ChunkIjkVector>,
    bottom_right: Option<ChunkIjkVector>,
}

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
    fn get_next_targets(&self) -> Vec<ChunkIjkVector> {
        let mut out = Vec::new();
        for j in (0..self
            .coords
            .get_total_number_chunks_in_concentric_circle_dimension())
            .skip((self.process_count / 3) % 3)
            .step_by(3)
        {
            let (layer_num, chunk_layer_concentric_circle) = self
                .coords
                .get_layer_num_from_absolute_chunk_concentric_circle(j);
            for k in (0..self.coords.get_chunk_layer_num_radial_lines(layer_num))
                .skip(self.process_count % 3)
                .step_by(3)
            {
                out.push(ChunkIjkVector {
                    i: layer_num,
                    j: chunk_layer_concentric_circle,
                    k,
                });
            }
        }
        out
    }

    // TODO: This needs testing
    fn get_chunk_top_neighbors(&self, coord: ChunkIjkVector) -> TopNeighbors {
        let top_chunk_in_layer = self.coords.get_chunk_layer_num_concentric_circles(coord.i);
        let top_layer = self.coords.get_num_layers() - 1;
        let radial_lines = |i: usize| self.coords.get_chunk_layer_num_radial_lines(i);
        let k_isize = coord.k as isize;

        let make_vector = |i: usize, j: usize, k: isize| -> ChunkIjkVector {
            ChunkIjkVector {
                i,
                j,
                k: modulo(k, radial_lines(i) as isize) as usize,
            }
        };

        // Default neighbors (middle of stuff)
        let default_neighbors = || -> TopNeighbors {
            TopNeighbors {
                top_left: Some(make_vector(coord.i, coord.j + 1, k_isize - 1)),
                top_center_1: Some(make_vector(coord.i, coord.j + 1, k_isize)),
                top_center_2: Some(make_vector(coord.i, coord.j + 1, k_isize + 1)),
                top_right: Some(make_vector(coord.i, coord.j + 1, k_isize + 2)),
            }
        };

        match (coord.i, coord.j) {
            (i, _) if i == top_layer => match coord.j {
                j if j == top_chunk_in_layer => TopNeighbors {
                    top_left: None,
                    top_center_1: None,
                    top_right: None,
                    top_center_2: None,
                },
                _ => default_neighbors(),
            },
            (_, j) if j == top_chunk_in_layer => TopNeighbors {
                top_left: Some(make_vector(coord.i + 1, 0, k_isize * 2 - 1)),
                top_center_1: Some(make_vector(coord.i + 1, 0, k_isize * 2)),
                top_center_2: Some(make_vector(coord.i + 1, 0, k_isize * 2 + 1)),
                top_right: Some(make_vector(coord.i + 1, 0, k_isize * 2 + 2)),
            },
            _ => default_neighbors(),
        }
    }

    // TODO: This needs testing
    fn get_chunk_left_right_neighbors(&self, coord: ChunkIjkVector) -> LeftRightNeighbors {
        let left = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(
                coord.k as isize - 1,
                self.coords.get_chunk_layer_num_radial_lines(coord.i) as isize,
            ) as usize,
        };
        let right = ChunkIjkVector {
            i: coord.i,
            j: coord.j,
            k: modulo(
                coord.k as isize + 1,
                self.coords.get_chunk_layer_num_radial_lines(coord.i) as isize,
            ) as usize,
        };
        LeftRightNeighbors { left, right }
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

        // Default neighbors
        let default_neighbors = || -> BottomNeighbors {
            BottomNeighbors {
                bottom_left: Some(make_vector(coord.i, coord.j - 1, k_isize + 1)),
                bottom_center: Some(make_vector(coord.i, coord.j - 1, k_isize)),
                bottom_right: Some(make_vector(coord.i, coord.j - 1, k_isize - 1)),
            }
        };

        match (coord.i, coord.j) {
            (i, j) if i == bottom_layer && j == bottom_chunk_in_layer => BottomNeighbors {
                bottom_left: None,
                bottom_center: None,
                bottom_right: None,
            },
            // If going down a layer but you are not at the bottom
            (i, j) if j == bottom_chunk_in_layer => BottomNeighbors {
                bottom_left: Some(make_vector(
                    coord.i - 1,
                    top_chunk_in_prev_layer(i),
                    k_isize / 2 + 1,
                )),
                bottom_center: None,
                bottom_right: Some(make_vector(
                    coord.i - 1,
                    top_chunk_in_prev_layer(i),
                    k_isize / 2,
                )), // This is not -1 because integer division naturally rounds down
            },
            _ => default_neighbors(),
        }
    }

    fn get_chunk_neighbors(&self, coord: ChunkIjkVector) -> ElementGridConvolutionChunkIdx {
        let top = self.get_chunk_top_neighbors(coord);
        let lr = self.get_chunk_left_right_neighbors(coord);
        let bottom = self.get_chunk_bottom_neighbors(coord);
        ElementGridConvolutionChunkIdx {
            tl: top.top_left,
            t1: top.top_center_1,
            t2: top.top_center_2,
            tr: top.top_right,
            l: lr.left,
            r: lr.right,
            bl: bottom.bottom_left,
            b: bottom.bottom_center,
            br: bottom.bottom_right,
        }
    }

    fn package_this_convolution(&mut self, coord: ChunkIjkVector) -> ElementGridConvolution {
        println!("Packaging convolution for chunk {:?}", coord);
        let neighbors = self.get_chunk_neighbors(coord);

        let t1 = neighbors
            .t1
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let t2 = neighbors
            .t2
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let tl = neighbors
            .tl
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let tr = neighbors
            .tr
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let l = std::mem::take(self.get_chunk_by_chunk_ijk_mut(neighbors.l));
        let r = std::mem::take(self.get_chunk_by_chunk_ijk_mut(neighbors.r));
        let bl = neighbors
            .bl
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let b = neighbors
            .b
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));
        let br = neighbors
            .br
            .map(|x| std::mem::take(self.get_chunk_by_chunk_ijk_mut(x)));

        ElementGridConvolution {
            t1,
            t2,
            tl,
            tr,
            l,
            r,
            bl,
            b,
            br,
        }
    }

    // This takes ownership of the chunk and all its neighbors from the directory
    // and puts them into a target vector and a vector of convolutions
    // The taget vector and convolution vectors will then be iterated on in parallel
    fn package_convolutions(&mut self) -> (Vec<ElementGridConvolution>, Vec<ElementGrid>) {
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
        convolutions: Vec<ElementGridConvolution>,
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
            for neighbor in this_conv.into_iter() {
                let neighbor_coord = neighbor.get_chunk_coords();
                let prev = self.chunks[neighbor_coord.get_layer_num()].replace(
                    JkVector {
                        j: neighbor_coord.get_start_concentric_circle_layer_relative(),
                        k: neighbor_coord.get_start_radial_line(),
                    },
                    Some(neighbor),
                );
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
