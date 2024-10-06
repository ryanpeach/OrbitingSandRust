use hashbrown::HashSet;

use super::{
    mesh::coordinate_dir::CoordinateDir,
    util::vectors::{IjkVector, JkVector, RelJkVector},
};

#[derive(Clone)]
pub struct PointCloud {
    points_by_layer: Vec<HashSet<JkVector>>,
}

impl PointCloud {
    pub fn new(num_layers: usize) -> Self {
        let mut points_by_layer = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            points_by_layer.push(HashSet::new());
        }
        PointCloud { points_by_layer }
    }

    pub fn insert(&mut self, point: IjkVector) {
        self.points_by_layer[point.i].insert(point.to_jk_vector());
    }

    pub fn expand_by(&self, n: u32, coord_dir: &CoordinateDir) -> PointCloud {
        let mut out = PointCloud::new(self.points_by_layer.len());
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
    /// let center = JkVector { j: 0, k: 0 };
    /// let points = get_square_points(&center, 3);
    /// // points will include all Vec2 from (-1, -1) to (1, 1)
    /// ```
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
}

struct JkRect {
    min: JkVector,
    max: JkVector,
}

struct Directory {
    rects_by_layer: Vec<Vec<JkRect>>,
}

impl Directory {
    pub fn new(point_cloud: &PointCloud) -> Directory {
        unimplemented!("Dirty rectangles implementation goes here")
    }
}
