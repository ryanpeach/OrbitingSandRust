use uom::si::time::second;

use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_convolution::ElementGridConvolution;
use super::element_grid::ElementGrid;
use super::util::{ChunkIjkVector, Grid, RawImage};

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
        let mut layers: Vec<Grid<Option<ElementGrid>>> =
            Vec::with_capacity(coords.get_num_layers());
        for i in 0..coords.len() {
            let j_size = coords.get_layer_num_concentric_circles(i);
            let k_size = coords.get_layer_num_radial_lines(i);
            let chunks = Vec::with_capacity(j_size * k_size);
            for j in 0..j_size {
                for k in 0..k_size {
                    chunks.push(ElementGrid::new_empty(
                        coords.get_chunk_by_chunk_ijk(ChunkIjkVector { i, j, k }),
                    ));
                }
            }
        }
        Self {
            coords,
            chunks,
            process_count: 0,
        }
    }

    fn package_this_convolution(&mut self, coord: ChunkIjkVector) -> ElementGridConvolution {
        // let t
        // ElementGridConvolution{
        //     t, tl, tr, l, r, bl, b, br
        // }
    }

    // TODO: This needs testing
    // This takes ownership of the chunk and all its neighbors from the directory
    // and puts them into a target vector and a vector of convolutions
    // The taget vector and convolution vectors will then be iterated on in parallel
    fn package_convolutions(&mut self) -> (Vec<ElementGridConvolution>, Vec<ElementGrid>) {
        let i_iter = (0..self.coords.get_num_layers())
            .skip(self.process_count % 2)
            .step_by(2);
        let mut convolutions = Vec::new();
        let mut target_chunks = Vec::new();
        for i in i_iter {
            let j_iter = (0..self.coords.get_layer_num_concentric_circles(i))
                .skip(self.process_count % 4)
                .step_by(2);

            let k_iter = (0..self.coords.get_layer_num_radial_lines(i))
                .skip(self.process_count % 4)
                .step_by(2);

            for (j, k) in j_iter.cartesian_product(k_iter) {
                let coord = ChunkIjkVector { i, j, k };
                let convolution = self.package_this_convolution(coord);
                convolutions.push(convolution);
                let prev = self.chunks[i].replace(coord.k, coord.j, None);
                debug_assert!(prev.is_some(), "Someone is already using this chunk!");
                target_chunks.push(prev.unwrap());
            }
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
        for (target_chunk, this_conv) in target_chunks.into_iter().zip(convolutions.into_iter()) {
            let coord = target_chunk.get_chunk_coords();
            let prev = self.chunks[coord.get_layer_num()].replace(
                coord.get_start_radial_line(),
                coord.get_start_concentric_circle_layer_relative(),
                Some(target_chunk),
            );
            debug_assert!(prev.is_none(), "Somehow this chunk was already replaced.");
            for neighbor in this_conv.into_iter() {
                let neighbor_coord = neighbor.get_chunk_coords();
                let prev = self.chunks[neighbor_coord.get_layer_num()].replace(
                    neighbor_coord.get_start_radial_line(),
                    neighbor_coord.get_start_concentric_circle_layer_relative(),
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
    fn process(&mut self, delta: second) {
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

    pub fn get_chunk_by_chunk_ijk(&self, coord: ChunkIjkVector) -> &ElementGrid {
        &self.chunks[coord.i].get(coord.k, coord.j).unwrap()
    }
    pub fn get_chunk_by_chunk_ijk_mut(&mut self, coord: ChunkIjkVector) -> &mut ElementGrid {
        &mut self.chunks[coord.i].get_mut(coord.k, coord.j).unwrap()
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
