//! implementation of dirty rectangles
//! Helps reduce computational load of processing falling sand by isolating
//! computation to the area around cells which changed last frame.
//!
//! Inspired by <https://youtu.be/prXuyMCgbTc?si=gDRGWQyRLmL151JL&t=631>

use crate::common::util::vectors::{
    ChunkIjkVector, IjkVector, InChunkJkVector, LayerJkVector, RelJkVector,
};

use super::mesh::{chunk_coords::ChunkCoords, coordinate_dir::CoordinateDir};
use conv::ValueFrom;
use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;
use std::collections::VecDeque;

/// All the points which changed along an entire celestial
#[derive(Clone)]
pub struct LayerPointClouds {
    /// Stores the last changed points in a layer with the layer number, over the entire celestial.
    pub points_by_layer: Vec<HashSet<LayerJkVector>>,
}

impl LayerPointClouds {
    /// Initialize
    #[must_use]
    pub fn new(num_layers: usize) -> Self {
        let mut points_by_layer = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            points_by_layer.push(HashSet::new());
        }
        LayerPointClouds { points_by_layer }
    }

    /// Insert a new point
    pub fn insert(&mut self, point: IjkVector) {
        self.points_by_layer[point.i as usize].insert(point.into());
    }

    /// For every point in the struct, create a new struct which also includes their neighbors, `n`
    /// around.
    #[must_use]
    pub fn expand_by(&self, n: u32, coord_dir: &CoordinateDir) -> LayerPointClouds {
        debug_assert!(n >= 1);
        let mut out = LayerPointClouds::new(self.points_by_layer.len());
        for (layer_num, layer) in self.points_by_layer.iter().enumerate() {
            for point in layer {
                for new_point in Self::get_square_points(point, n) {
                    for possible_conversion in coord_dir.fuzzy_rel_ijk_to_absolute_ijk(
                        u32::value_from(layer_num)
                            .expect("Number of layers is not very large, usually under 100."),
                        new_point,
                    ) {
                        out.insert(possible_conversion);
                    }
                }
            }
        }
        out
    }

    /// Returns all points within a square of width `n` around the given `center` point.
    ///
    /// # Arguments
    ///
    /// * `center` - A reference to the central Vec2 point.
    /// * `n` - The width of the square (must be a positive integer).
    ///
    /// # Returns
    ///
    /// A vector containing all Vec2 points within the specified square.
    ///
    /// # Example
    ///
    /// ```
    /// ```
    #[must_use]
    fn get_square_points(center: &LayerJkVector, n: u32) -> Vec<RelJkVector> {
        // Calculate half the width. If n is even, the square will be slightly asymmetric.
        let half_n = i64::from(n) / 2;

        let mut points = Vec::new();

        for dk in -half_n..=half_n {
            for dj in -half_n..=half_n {
                let point = RelJkVector {
                    rk: i64::from(center.k) + dk,
                    rj: i64::from(center.j) + dj,
                };
                points.push(point);
            }
        }

        points
    }

    /// Given a [`ChunkPointClouds`] object and a [`CoordinateDir`] assemble this object.
    /// It needs the coordinate dir to convert chunk indexes into [`IjkVector`]s.
    #[must_use]
    pub fn from_chunk_point_clouds(
        chunk_point_clouds: ChunkPointClouds,
        coord_dir: &CoordinateDir,
    ) -> Self {
        let mut out: Vec<HashSet<LayerJkVector>> =
            Vec::with_capacity(coord_dir.num_layers() as usize);
        for _ in 0..coord_dir.num_layers() {
            out.push(HashSet::new());
        }
        for (chunk_idx, in_set) in chunk_point_clouds.points_by_chunk {
            for point in in_set {
                let this = coord_dir
                    .chunk_at_idx(chunk_idx)
                    .external_coord_from_internal_coord(point);
                out[this.i as usize].insert(this.into());
            }
        }
        Self {
            points_by_layer: out,
        }
    }
}

/// Just like [`LayerPointClouds`] except its indexed by chunk.
/// This is the first object assembled during processing
/// This is later transformed into [`LayerPointClouds`] via
/// [`LayerPointClouds::from_chunk_point_clouds`]
#[derive(Default, Debug)]
pub struct ChunkPointClouds {
    /// Maps the chunks index in the celestial to the set of in chunk positions which changed last
    /// time you called process.
    pub points_by_chunk: HashMap<ChunkIjkVector, HashSet<InChunkJkVector>>,
}

impl From<HashMap<ChunkIjkVector, HashSet<InChunkJkVector>>> for ChunkPointClouds {
    fn from(value: HashMap<ChunkIjkVector, HashSet<InChunkJkVector>>) -> Self {
        Self {
            points_by_chunk: value,
        }
    }
}

impl ChunkPointClouds {
    /// Used to convert [`LayerPointClouds`] **back** into [`ChunkPointClouds`]
    /// We do this to get the rectangles back into a container we can associate with a single chunk
    #[must_use]
    pub fn from_layer_point_clouds(
        layer_point_clouds: LayerPointClouds,
        coord_dir: &CoordinateDir,
    ) -> Self {
        let mut out: HashMap<ChunkIjkVector, HashSet<InChunkJkVector>> = HashMap::new();
        for (layer_num, layer) in layer_point_clouds.points_by_layer.iter().enumerate() {
            for point in layer {
                let cell_idx = IjkVector {
                    i: u32::value_from(layer_num)
                        .expect("Number of layers is usually less than 100"),
                    j: point.j,
                    k: point.k,
                };
                let full_idx = coord_dir.cell_idx_to_full_idx(cell_idx);
                if let Some(set) = out.get_mut(&full_idx.chunk_idx) {
                    set.insert(full_idx.pos);
                } else {
                    let mut new_set = HashSet::new();
                    new_set.insert(full_idx.pos);
                    out.insert(full_idx.chunk_idx, new_set);
                }
            }
        }
        ChunkPointClouds {
            points_by_chunk: out,
        }
    }
}

/// A rectangle in Jk Coordinates.
/// This one is "`InChunk`"
pub struct JkRect {
    /// minimum j coordinate
    pub min_j: usize,
    /// maximum j coordinate
    pub max_j: usize,
    /// minimum k cordinate
    pub min_k: usize,
    /// maximum k coordinate
    pub max_k: usize,
}

/// A set of all rectangles
pub struct Directory {
    /// Organizes the rectangles by chunk
    pub rects_by_chunk: HashMap<ChunkIjkVector, Vec<JkRect>>,
}

impl Directory {
    /// From a chunk point cloud, assemble all rectangles
    #[must_use]
    pub fn new(chunk_point_clouds: ChunkPointClouds, coord_dir: &CoordinateDir) -> Directory {
        let rects_by_chunk: HashMap<ChunkIjkVector, Vec<JkRect>> = chunk_point_clouds
            .points_by_chunk
            .par_iter()
            .map(|(chunk_idx, set)| {
                let rects =
                    Directory::calc_dirty_rects(set.clone(), &coord_dir.chunk_at_idx(*chunk_idx));
                (*chunk_idx, rects)
            })
            .collect();

        Directory { rects_by_chunk }
    }

    /// Helper for [`Self::new`]
    /// The implementation of the dirty rectangles algorithm on a set of points.
    #[must_use]
    fn calc_dirty_rects(
        point_cloud: HashSet<InChunkJkVector>,
        chunk_coords: &ChunkCoords,
    ) -> Vec<JkRect> {
        let mut rects = Vec::new();
        let mut visited = HashSet::with_capacity(point_cloud.len());
        let mut queue = VecDeque::new();

        for point in &point_cloud {
            if !visited.contains(point) {
                // Start BFS
                queue.push_back(*point);
                visited.insert(*point);

                // Initialize bounding box with the first point
                let mut min_j = point.j;
                let mut max_j = point.j;
                let mut min_k = point.k;
                let mut max_k = point.k;

                while let Some(current) = queue.pop_front() {
                    // Update bounding box
                    if current.j < min_j {
                        min_j = current.j;
                    }
                    if current.j > max_j {
                        max_j = current.j;
                    }
                    if current.k < min_k {
                        min_k = current.k;
                    }
                    if current.k > max_k {
                        max_k = current.k;
                    }

                    // Define 8-connected neighbors
                    let neighbors: Vec<InChunkJkVector> = [
                        RelJkVector {
                            rj: i64::from(current.j) + 1,
                            rk: i64::from(current.k) + 1,
                        },
                        RelJkVector {
                            rj: i64::from(current.j) + 1,
                            rk: i64::from(current.k),
                        },
                        RelJkVector {
                            rj: i64::from(current.j) + 1,
                            rk: i64::from(current.k) - 1,
                        },
                        RelJkVector {
                            rj: i64::from(current.j),
                            rk: i64::from(current.k) + 1,
                        },
                        RelJkVector {
                            rj: i64::from(current.j),
                            rk: i64::from(current.k) - 1,
                        },
                        RelJkVector {
                            rj: i64::from(current.j) - 1,
                            rk: i64::from(current.k) + 1,
                        },
                        RelJkVector {
                            rj: i64::from(current.j) - 1,
                            rk: i64::from(current.k),
                        },
                        RelJkVector {
                            rj: i64::from(current.j) - 1,
                            rk: i64::from(current.k) - 1,
                        },
                    ]
                    // Eliminate anything out of bounds
                    .into_iter()
                    .filter(|x| {
                        x.rj >= 0
                            && x.rk >= 0
                            && x.rj < i64::from(chunk_coords.num_concentric_circles())
                            && x.rk < i64::from(chunk_coords.num_radial_lines())
                    })
                    .map(derive_more::Into::into)
                    .collect();

                    for neighbor in &neighbors {
                        if point_cloud.contains(neighbor) && !visited.contains(neighbor) {
                            queue.push_back(*neighbor);
                            visited.insert(*neighbor);
                        }
                    }
                }

                // After BFS, create the bounding rectangle
                let rect = JkRect {
                    min_j: min_j as usize,
                    max_j: max_j as usize,
                    min_k: min_k as usize,
                    max_k: max_k as usize,
                };
                rects.push(rect);
            }
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use crate::common::util::vectors::RelJkVector;

    use super::LayerPointClouds;

    /// Test a simple 3x3 grid around the origin
    #[test]
    fn test_calc_square_points() {
        let center = crate::common::util::vectors::LayerJkVector { j: 0, k: 0 };
        let points = LayerPointClouds::get_square_points(&center, 3);
        assert!(points.contains(&RelJkVector { rj: -1, rk: -1 }));
        assert!(points.contains(&RelJkVector { rj: -1, rk: 0 }));
        assert!(points.contains(&RelJkVector { rj: -1, rk: 1 }));
        assert!(points.contains(&RelJkVector { rj: 0, rk: -1 }));
        assert!(points.contains(&RelJkVector { rj: 0, rk: 0 }));
        assert!(points.contains(&RelJkVector { rj: 0, rk: 1 }));
        assert!(points.contains(&RelJkVector { rj: 1, rk: -1 }));
        assert!(points.contains(&RelJkVector { rj: 1, rk: 0 }));
        assert!(points.contains(&RelJkVector { rj: 1, rk: 1 }));
        assert_eq!(points.len(), 9);
    }
}
