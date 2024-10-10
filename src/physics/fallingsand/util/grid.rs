//! A simple 2d grid type
//! This was originally created seperate from the ndarray crate, but it was later decided to
//! use the ndarray crate as the backend for this type. This is because the ndarray crate
//! has a convolution function that is helpful for the physics simulation.
//! So some of this code is now redundant, but is maintained for legacy reasons

use ndarray::ShapeError;
use thiserror::Error;

use super::vectors::JkVector;
use conv::ValueFrom;

/// A simple 2d grid type
#[derive(Clone, Debug)]
pub struct Grid<T>(ndarray::Array2<T>);

/* =================
 * Initialization
 * ================= */
impl<T> Grid<T> {
    /// Create a new grid filled with one value
    pub fn new_fill(width: u32, height: u32, value: T) -> Self
    where
        T: Clone,
    {
        Self(ndarray::Array2::from_elem(
            (width as usize, height as usize),
            value,
        ))
    }
    /// Create a new grid with the given width and height, and fill it with the given data
    pub fn new_from_vec(width: u32, height: u32, data: Vec<T>) -> Result<Self, ShapeError> {
        Ok(Self(ndarray::Array2::from_shape_vec(
            (width as usize, height as usize),
            data,
        )?))
    }
    /// Create a new grid with the given width and height, and fill it with default values
    #[must_use]
    pub fn new_empty(width: u32, height: u32) -> Self
    where
        T: Default,
    {
        let mut data = Vec::with_capacity((width * height) as usize);
        for _ in 0..width * height {
            data.push(Default::default());
        }
        Self(
            ndarray::Array2::from_shape_vec((width as usize, height as usize), data)
                .expect("We made data ourselves."),
        )
    }
}

/* ======================================
 * Simple Getters
 * Access basic attributes of the struct
 * ====================================== */
impl<T> Grid<T> {
    /// Get the width of the grid
    #[must_use]
    pub fn width(&self) -> u32 {
        u32::value_from(self.0.shape()[0]).expect("Initialization uses u32")
    }
    /// Get the height of the grid
    #[must_use]
    pub fn height(&self) -> u32 {
        u32::value_from(self.0.shape()[1]).expect("Initialization uses u32")
    }
    /// Get the total size of the grid
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.0.len()
    }
    /// Get the data as a slice
    #[must_use]
    pub fn data_slice(&self) -> &[T] {
        self.0
            .as_slice()
            .expect("TODO: In what case is a slice non-standard?")
    }
    /// Get the data as an ndarray
    #[must_use]
    pub fn data(&self) -> &ndarray::Array2<T> {
        &self.0
    }
}

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone, Error)]
#[error("{:?} went outside the constraints of grid", .0)]
pub struct GridOutOfBoundsError(pub JkVector);

/* ======================================
 * Position Based Getters
 * Access data at a position
 * ====================================== */
/// Access data using JK coordinates, which are height and width respectively
impl<T> Grid<T> {
    /// Gets the value at the given coordinate
    ///
    /// # Panics
    /// - if idx is out of index
    ///
    /// TODO: More to convention, use square brackets
    /// TODO: Get should return Option
    #[must_use]
    pub fn get(&self, idx: JkVector) -> &T {
        let idx = self.transform_jk_coord_to_ndarray(idx);
        &self.0[idx]
    }
    /// Gets the value at the given coordinate, or returns an error if the coordinate is out of bounds
    ///
    /// TODO: Rename to get, and change to option
    pub fn checked_get(&self, idx: JkVector) -> Result<&T, GridOutOfBoundsError> {
        if idx.k >= self.width() || idx.j >= self.height() {
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
        [
            self.width() as usize - 1 - idx.k as usize,
            self.height() as usize - 1 - idx.j as usize,
        ]
    }
}

/// Iteration
impl<T> Grid<T> {
    /// Get an iterator over the grid
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.0
            .as_slice()
            .expect("TODO: In what cases is a slice non-standard?")
            .iter()
    }

    /// Get a mutable iterator over the grid
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.0
            .as_slice_mut()
            .expect("TODO: In what cases is a slice non-standard?")
            .iter_mut()
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
#[must_use]
pub fn filter_vecgrid<T>(grid: &[Grid<T>], filter: &[Grid<bool>]) -> Vec<Grid<T>>
where
    T: Default + Clone,
{
    let mut out = Vec::new();
    for (i, item) in filter.iter().enumerate() {
        let j_size = item.height();
        let k_size = item.width();
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
    #![allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::unwrap_used,
        clippy::panic
    )]
    use super::*;

    #[test]
    fn test_iter() {
        let grid = Grid::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]).unwrap();
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
        let mut grid = Grid::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]).unwrap();

        for val in &mut grid {
            *val *= 2;
        }

        assert_eq!(grid.data_slice(), &[2, 4, 6, 8, 10, 12]);
    }
}
