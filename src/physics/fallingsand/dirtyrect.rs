use super::{
    mesh::{chunk_coords::ChunkCoords, coordinate_dir::CoordinateDir},
    util::vectors::{ChunkIjkVector, IjkVector, JkVector, RelJkVector},
};
use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct LayerPointClouds {
    pub points_by_layer: Vec<HashSet<JkVector>>,
}

impl LayerPointClouds {
    #[must_use]
    pub fn new(num_layers: usize) -> Self {
        let mut points_by_layer = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            points_by_layer.push(HashSet::new());
        }
        LayerPointClouds { points_by_layer }
    }

    pub fn insert(&mut self, point: IjkVector) {
        self.points_by_layer[point.i].insert(point.into());
    }

    #[must_use]
    pub fn expand_by(&self, n: u32, coord_dir: &CoordinateDir) -> LayerPointClouds {
        debug_assert!(n >= 1);
        let mut out = LayerPointClouds::new(self.points_by_layer.len());
        for (layer_num, layer) in self.points_by_layer.iter().enumerate() {
            for point in layer {
                for new_point in Self::get_square_points(point, n) {
                    for possible_conversion in
                        coord_dir.fuzzy_rel_ijk_to_absolute_ijk(layer_num, new_point)
                    {
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
    fn get_square_points(center: &JkVector, n: u32) -> Vec<RelJkVector> {
        // Calculate half the width. If n is even, the square will be slightly asymmetric.
        let half_n = (n as isize) / 2;

        let mut points = Vec::new();

        for dk in -half_n..=half_n {
            for dj in -half_n..=half_n {
                let point = RelJkVector {
                    rk: center.k as isize + dk,
                    rj: center.j as isize + dj,
                };
                points.push(point);
            }
        }

        points
    }

    #[must_use]
    pub fn from_chunk_point_clouds(
        chunk_point_clouds: ChunkPointClouds,
        coord_dir: &CoordinateDir,
    ) -> Self {
        let mut out: Vec<HashSet<JkVector>> = Vec::with_capacity(coord_dir.num_layers());
        for _ in 0..coord_dir.num_layers() {
            out.push(HashSet::new());
        }
        for (chunk_idx, in_set) in chunk_point_clouds.points_by_chunk {
            for point in in_set {
                let this = coord_dir
                    .chunk_at_idx(chunk_idx)
                    .external_coord_from_internal_coord(point);
                out[this.i].insert(this.into());
            }
        }
        Self {
            points_by_layer: out,
        }
    }
}

#[derive(Default, Debug)]
pub struct ChunkPointClouds {
    pub points_by_chunk: HashMap<ChunkIjkVector, HashSet<JkVector>>,
}

impl From<HashMap<ChunkIjkVector, HashSet<JkVector>>> for ChunkPointClouds {
    fn from(value: HashMap<ChunkIjkVector, HashSet<JkVector>>) -> Self {
        Self {
            points_by_chunk: value,
        }
    }
}

impl ChunkPointClouds {
    #[must_use]
    pub fn from_layer_point_clouds(
        layer_point_clouds: LayerPointClouds,
        coord_dir: &CoordinateDir,
    ) -> Self {
        let mut out: HashMap<ChunkIjkVector, HashSet<JkVector>> = HashMap::new();
        for (layer_num, layer) in layer_point_clouds.points_by_layer.iter().enumerate() {
            for point in layer {
                let cell_idx = IjkVector {
                    i: layer_num,
                    j: point.j,
                    k: point.k,
                };
                let (chunk_idx, cell_idx_inside_chunk) = coord_dir.cell_idx_to_chunk_idx(cell_idx);
                match out.get_mut(&chunk_idx) {
                    Some(set) => {
                        set.insert(cell_idx_inside_chunk);
                    }
                    None => {
                        let mut new_set = HashSet::new();
                        new_set.insert(cell_idx_inside_chunk);
                        out.insert(chunk_idx, new_set);
                    }
                }
            }
        }
        ChunkPointClouds {
            points_by_chunk: out,
        }
    }
}

pub struct JkRect {
    pub min_j: usize,
    pub max_j: usize,
    pub min_k: usize,
    pub max_k: usize,
}

pub struct Directory {
    pub rects_by_chunk: HashMap<ChunkIjkVector, Vec<JkRect>>,
}

impl Directory {
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

    #[must_use]
    pub fn calc_dirty_rects(
        point_cloud: HashSet<JkVector>,
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
                    let neighbors: Vec<JkVector> = [
                        RelJkVector {
                            rj: current.j as isize + 1,
                            rk: current.k as isize + 1,
                        },
                        RelJkVector {
                            rj: current.j as isize + 1,
                            rk: current.k as isize,
                        },
                        RelJkVector {
                            rj: current.j as isize + 1,
                            rk: current.k as isize - 1,
                        },
                        RelJkVector {
                            rj: current.j as isize,
                            rk: current.k as isize + 1,
                        },
                        RelJkVector {
                            rj: current.j as isize,
                            rk: current.k as isize - 1,
                        },
                        RelJkVector {
                            rj: current.j as isize - 1,
                            rk: current.k as isize + 1,
                        },
                        RelJkVector {
                            rj: current.j as isize - 1,
                            rk: current.k as isize,
                        },
                        RelJkVector {
                            rj: current.j as isize - 1,
                            rk: current.k as isize - 1,
                        },
                    ]
                    // Eliminate anything out of bounds
                    .into_iter()
                    .filter(|x| {
                        x.rj >= 0
                            && x.rk >= 0
                            && x.rj < chunk_coords.num_concentric_circles() as isize
                            && x.rk < chunk_coords.num_radial_lines() as isize
                    })
                    .map(|x| x.into())
                    .collect();

                    for neighbor in neighbors.iter() {
                        if point_cloud.contains(neighbor) && !visited.contains(neighbor) {
                            queue.push_back(*neighbor);
                            visited.insert(*neighbor);
                        }
                    }
                }

                // After BFS, create the bounding rectangle
                let rect = JkRect {
                    min_j,
                    max_j,
                    min_k,
                    max_k,
                };
                rects.push(rect);
            }
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::LayerPointClouds;
    use crate::physics::fallingsand::util::vectors::{JkVector, RelJkVector};

    /// Test a simple 3x3 grid around the origin
    #[test]
    fn test_calc_square_points() {
        let center = JkVector { j: 0, k: 0 };
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
