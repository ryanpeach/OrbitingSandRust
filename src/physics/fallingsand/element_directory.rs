use uom::si::f64::Time;
use uom::si::time::second;

use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_convolution::{ElementGridConvolution, ElementGridConvolutionChunkIdx};
use super::element_grid::ElementGrid;
use super::util::grid::Grid;
use super::util::image::RawImage;
use super::util::vectors::{ChunkIjkVector, JkVector};

use itertools::Itertools;
use rayon::prelude::*;

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
            let j_size = coords.get_layer_num_concentric_circles(i);
            let k_size = coords.get_layer_num_radial_lines(i);
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
    fn get_next_targets(&self) -> Vec<ChunkIjkVector> {}

    // TODO: This needs testing
    fn get_neighbors(&self, coord: ChunkIjkVector) -> ElementGridConvolutionChunkIdx {}

    /// Packages the neighbors of a chunk into a convolution object
    fn package_this_convolution(&mut self, coord: ChunkIjkVector) -> ElementGridConvolution {
        let neighbors = self.get_neighbors(coord);
        let t1: Option<ElementGrid> = match neighbors.t1 {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let t2: Option<ElementGrid> = match neighbors.t2 {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let tl: Option<ElementGrid> = match neighbors.tl {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let tr: Option<ElementGrid> = match neighbors.tr {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let l: ElementGrid = *self.get_chunk_by_chunk_ijk(neighbors.l);
        let r: ElementGrid = *self.get_chunk_by_chunk_ijk(neighbors.r);
        let bl: Option<ElementGrid> = match neighbors.bl {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let b: Option<ElementGrid> = match neighbors.b {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
        let br: Option<ElementGrid> = match neighbors.br {
            Some(x) => Some(*self.get_chunk_by_chunk_ijk(x)),
            None => None,
        };
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
        let convolutions = target_chunk_coords
            .into_par_iter()
            .map(|coord| self.package_this_convolution(coord))
            .collect();
        let target_chunks = target_chunk_coords
            .into_par_iter()
            .map(|coord| {
                self.chunks[coord.i]
                    .replace(coord.to_jk_vector(), None)
                    .unwrap()
            })
            .collect();
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
        for (target_chunk, this_conv) in target_chunks.into_iter().zip(convolutions.into_iter()) {
            let coord = target_chunk.get_chunk_coords();
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

    /// Do one iteration of processing on the grid
    /// There are four passes in total, each call does one pass
    /// The passes ensure that no two adjacent elementgrids are processed at the same time
    /// This is important because elementgrids can effect one another at a maximum range of
    /// the size of one elementgrid.
    pub fn process(&mut self, delta: Time) {
        let (mut convolutions, mut target_chunks) = self.package_convolutions();
        convolutions
            .into_par_iter()
            .zip(target_chunks.into_par_iter())
            .for_each(|(mut convolution, mut target_chunk)| {
                target_chunk.process(&mut convolution, delta);
            });
        self.unpackage_convolutions(convolutions, target_chunks);
        self.process_count += 1;
    }

    pub fn get_num_chunks(&self) -> usize {
        self.coords.get_num_chunks()
    }

    /// Gets the chunk at the given index
    /// Errors if it is currently borrowed
    pub fn get_chunk_by_chunk_ijk(&self, coord: ChunkIjkVector) -> &ElementGrid {
        &self.chunks[coord.i].get(coord.to_jk_vector()).unwrap()
    }
    /// Gets the chunk at the given index mutably
    /// Errors if it is currently borrowed
    pub fn get_chunk_by_chunk_ijk_mut(&mut self, coord: ChunkIjkVector) -> &mut ElementGrid {
        &mut self.chunks[coord.i].get_mut(coord.to_jk_vector()).unwrap()
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
                let j_range = 0..self.coords.get_layer_num_concentric_circles(i);
                let k_range = 0..self.coords.get_layer_num_radial_lines(i);
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
