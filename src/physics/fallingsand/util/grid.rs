use super::vectors::JkVector;

/// A simple 2d grid type
/// Just so we don't have to download a new crate
#[derive(Clone)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

/* =================
 * Initialization
 * ================= */
impl<T> Grid<T> {
    pub fn new(width: usize, height: usize, data: Vec<T>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
    pub fn new_empty(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        let mut data = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            data.push(Default::default());
        }
        Self {
            width,
            height,
            data,
        }
    }
}

/* ======================================
 * Simple Getters
 * Access basic attributes of the struct
 * ====================================== */
impl<T> Grid<T> {
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn total_size(&self) -> usize {
        self.data.len()
    }
    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }
}

/* ======================================
 * Position Based Getters
 * Access data at a position
 * ====================================== */
impl<T> Grid<T> {
    pub fn get(&self, coord: JkVector) -> &T {
        &self.data[coord.k + coord.j * self.width]
    }
    pub fn get_mut(&mut self, coord: JkVector) -> &mut T {
        &mut self.data[coord.k + coord.j * self.width]
    }
    /// Like get, but gives you ownership of the value and replaces it with the replacement
    pub fn replace(&mut self, coord: JkVector, replacement: T) -> T {
        let idx = coord.k + coord.j * self.width;
        std::mem::replace(&mut self.data[idx], replacement)
    }
}

/* ==================================
 * Coordinate Transforms
 * ================================== */
impl<T> Grid<T> {
    /// Convert the flat vector coordinate to the grid specific coordinate
    /// Opposite of jk_coord_to_flat_idx
    pub fn flat_idx_to_jk_coord(&self, flat_idx: usize) -> JkVector {
        let j = flat_idx / self.width;
        let k = flat_idx % self.width;
        JkVector { j, k }
    }

    /// Convert the grid specific coordinate to a flat vector coordinate
    /// Opposite of flat_idx_to_jk_coord
    pub fn jk_coord_to_flat_idx(&self, coord: JkVector) -> usize {
        coord.j * self.width + coord.k
    }
}

/* Iteration */
impl<T> Grid<T> {
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
}
impl<'a, T> IntoIterator for &'a Grid<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let mut iter = grid.iter();

        assert_eq!(*iter.next().unwrap(), 1);
        assert_eq!(*iter.next().unwrap(), 2);
        assert_eq!(*iter.next().unwrap(), 3);
        assert_eq!(*iter.next().unwrap(), 4);
        assert_eq!(*iter.next().unwrap(), 5);
        assert_eq!(*iter.next().unwrap(), 6);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iter_mut() {
        let mut grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        for val in grid.iter_mut() {
            *val *= 2;
        }

        assert_eq!(grid.data, vec![2, 4, 6, 8, 10, 12]);
    }

    #[test]
    fn test_flat_idx_to_jk_coord() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        assert_eq!(grid.flat_idx_to_jk_coord(0), JkVector { j: 0, k: 0 });
        assert_eq!(grid.flat_idx_to_jk_coord(1), JkVector { j: 0, k: 1 });
        assert_eq!(grid.flat_idx_to_jk_coord(2), JkVector { j: 1, k: 0 });
        assert_eq!(grid.flat_idx_to_jk_coord(4), JkVector { j: 2, k: 0 });
    }

    #[test]
    fn test_jk_coord_to_flat_idx() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        assert_eq!(grid.jk_coord_to_flat_idx(JkVector { j: 0, k: 0 }), 0);
        assert_eq!(grid.jk_coord_to_flat_idx(JkVector { j: 0, k: 1 }), 1);
        assert_eq!(grid.jk_coord_to_flat_idx(JkVector { j: 1, k: 0 }), 2);
        assert_eq!(grid.jk_coord_to_flat_idx(JkVector { j: 2, k: 0 }), 4);
    }

    #[test]
    fn test_coord_conversion_inverse() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        for flat_idx in 0..grid.total_size() {
            let coord = grid.flat_idx_to_jk_coord(flat_idx);
            assert_eq!(grid.jk_coord_to_flat_idx(coord), flat_idx);
        }
    }

    #[test]
    fn test_flat_idx_conversion_inverse() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        let coords = [
            JkVector { j: 0, k: 0 },
            JkVector { j: 0, k: 1 },
            JkVector { j: 1, k: 0 },
            JkVector { j: 1, k: 1 },
            JkVector { j: 2, k: 0 },
            JkVector { j: 2, k: 1 },
        ];

        for &coord in coords.iter() {
            let flat_idx = grid.jk_coord_to_flat_idx(coord);
            assert_eq!(grid.flat_idx_to_jk_coord(flat_idx), coord);
        }
    }

    #[test]
    fn test_coord_flat_idx_consistency() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        for flat_idx in 0..grid.total_size() {
            let coord = grid.flat_idx_to_jk_coord(flat_idx);
            let direct_access = &grid.get_data()[flat_idx];
            let coord_access = grid.get(coord);
            assert_eq!(
                direct_access, coord_access,
                "Mismatch at flat_idx: {}",
                flat_idx
            );
        }
    }

    #[test]
    fn test_flat_idx_coord_consistency() {
        let grid = Grid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);

        let coords = [
            JkVector { j: 0, k: 0 },
            JkVector { j: 0, k: 1 },
            JkVector { j: 1, k: 0 },
            JkVector { j: 1, k: 1 },
            JkVector { j: 2, k: 0 },
            JkVector { j: 2, k: 1 },
        ];

        for &coord in coords.iter() {
            let flat_idx = grid.jk_coord_to_flat_idx(coord);
            let direct_access = &grid.get_data()[flat_idx];
            let coord_access = grid.get(coord);
            assert_eq!(
                direct_access, coord_access,
                "Mismatch at coord: ({}, {})",
                coord.j, coord.k
            );
        }
    }
}
