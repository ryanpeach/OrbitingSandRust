//! A simple 2d grid type
//! This was originally created seperate from the ndarray crate, but it was later decided to
//! use the ndarray crate as the backend for this type. This is because the ndarray crate
//! has a convolution function that is helpful for the physics simulation.
//! So some of this code is now redundant, but is maintained for legacy reasons
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::fmt;

use super::vectors::JkVector;

/// A simple 2d grid type
#[derive(Clone)]
pub struct Grid<T>(ndarray::Array2<T>);

/* =================
 * Initialization
 * ================= */
impl<T> Grid<T> {
    /// Create a new grid filled with one value
    pub fn new_fill(width: usize, height: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self(ndarray::Array2::from_elem((width, height), value))
    }
    /// Create a new grid with the given width and height, and fill it with the given data
    pub fn new_from_vec(width: usize, height: usize, data: Vec<T>) -> Self {
        Self(ndarray::Array2::from_shape_vec((width, height), data).unwrap())
    }
    /// Create a new grid with the given width and height, and fill it with default values
    pub fn new_empty(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        let mut data = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            data.push(Default::default());
        }
        Self(ndarray::Array2::from_shape_vec((width, height), data).unwrap())
    }
}

/* ======================================
 * Simple Getters
 * Access basic attributes of the struct
 * ====================================== */
impl<T> Grid<T> {
    /// Get the width of the grid
    pub fn get_width(&self) -> usize {
        self.0.shape()[0]
    }
    /// Get the height of the grid
    pub fn get_height(&self) -> usize {
        self.0.shape()[1]
    }
    /// Get the total size of the grid
    pub fn total_size(&self) -> usize {
        self.0.len()
    }
    /// Get the data as a slice
    pub fn get_data_slice(&self) -> &[T] {
        self.0.as_slice().unwrap()
    }
    /// Get the data as an ndarray
    pub fn get_data(&self) -> &ndarray::Array2<T> {
        &self.0
    }
}

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone)]
pub struct GridOutOfBoundsError(pub JkVector);
impl fmt::Display for GridOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} went outside the constraints of grid", self.0)
    }
}

/* ======================================
 * Position Based Getters
 * Access data at a position
 * ====================================== */
/// Access data using JK coordinates, which are height and width respectively
impl<T> Grid<T> {
    /// Gets the value at the given coordinate
    pub fn get(&self, idx: JkVector) -> &T {
        let idx = self.transform_jk_coord_to_ndarray(idx);
        &self.0[idx]
    }
    /// Gets the value at the given coordinate, or returns an error if the coordinate is out of bounds
    pub fn checked_get(&self, idx: JkVector) -> Result<&T, GridOutOfBoundsError> {
        if idx.k >= self.get_width() || idx.j >= self.get_height() {
            return Err(GridOutOfBoundsError(idx));
        }
        Ok(self.get(idx))
    }
    /// Gets the value at the given coordinate, mutably
    pub fn get_mut(&mut self, idx: JkVector) -> &mut T {
        let idx = self.transform_jk_coord_to_ndarray(idx);
        &mut self.0[idx]
    }
    /// Sets the value at the given coordinate, overwriting the old value
    pub fn set(&mut self, idx: JkVector, value: T) {
        self.replace(idx, value);
    }
    /// Like set, but gives you ownership of the original value
    pub fn replace(&mut self, idx: JkVector, replacement: T) -> T {
        let coord = self.transform_jk_coord_to_ndarray(idx);
        std::mem::replace(&mut self.0[coord], replacement)
    }
    /// Transforms the coordinate to the ndarray coordinate system using this grid's width and height
    fn transform_jk_coord_to_ndarray(&self, idx: JkVector) -> [usize; 2] {
        [self.get_width() - 1 - idx.k, self.get_height() - 1 - idx.j]
    }
}

/// Iteration
impl<T> Grid<T> {
    /// Get an iterator over the grid
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0.as_slice().unwrap().iter()
    }

    /// Get a mutable iterator over the grid
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.0.as_slice_mut().unwrap().iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a Grid<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Where filter is true, get the textures
pub fn filter_vecgrid<T>(grid: &[Grid<T>], filter: &[Grid<bool>]) -> Vec<Grid<T>>
where
    T: Default + Clone,
{
    let mut out = Vec::new();
    for (i, item) in filter.iter().enumerate() {
        let j_size = item.get_height();
        let k_size = item.get_width();
        let mut layer = Grid::new_empty(k_size, j_size);
        for j in 0..j_size {
            for k in 0..k_size {
                if *item.get(JkVector { j, k }) {
                    layer.set(JkVector { j, k }, grid[i].get(JkVector { j, k }).clone());
                }
            }
        }
        out.push(layer);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let grid = Grid::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]);
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
        let mut grid = Grid::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]);

        for val in grid.iter_mut() {
            *val *= 2;
        }

        assert_eq!(grid.get_data_slice(), &[2, 4, 6, 8, 10, 12]);
    }
}
