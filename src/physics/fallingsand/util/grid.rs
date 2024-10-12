//! A simple 2d grid type
//! This was originally created seperate from the ndarray crate, but it was later decided to
//! use the ndarray crate as the backend for this type. This is because the ndarray crate
//! has a convolution function that is helpful for the physics simulation.
//! So some of this code is now redundant, but is maintained for legacy reasons

use std::marker::PhantomData;

use ndarray::{Array2, ShapeError};
use thiserror::Error;

use crate::common::util::vectors::JkVector;
use conv::ValueFrom;

/// A simple 2d grid type
#[derive(Clone)]
pub struct JkGrid<T, V>
where
    T: JkVector,
{
    /// The actual array/grid/data
    data: Array2<V>,
    /// This is a trick to give [T] a definite use in "data"
    /// When really, [T] is just there to set our index type.
    phantom: PhantomData<T>,
}

/* =================
 * Initialization
 * ================= */
impl<T, V> JkGrid<T, V>
where
    T: JkVector,
    V: Clone,
{
    /// Create a new grid filled with one value
    pub fn new_fill(width: u32, height: u32, value: V) -> Self
    where
        T: Clone,
    {
        Self {
            data: ndarray::Array2::from_elem((width as usize, height as usize), value),
            phantom: PhantomData,
        }
    }
}
impl<T, V> JkGrid<T, V>
where
    T: JkVector,
{
    /// Create a new grid with the given width and height, and fill it with the given data
    pub fn new_from_vec(width: u32, height: u32, data: Vec<V>) -> Result<Self, ShapeError> {
        Ok(Self {
            data: ndarray::Array2::from_shape_vec((width as usize, height as usize), data)?,
            phantom: PhantomData,
        })
    }
}

impl<T, V> JkGrid<T, V>
where
    T: JkVector,
    V: Default,
{
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
        Self {
            data: ndarray::Array2::from_shape_vec((width as usize, height as usize), data)
                .expect("We made data ourselves."),
            phantom: PhantomData,
        }
    }
}

/* ======================================
 * Simple Getters
 * Access basic attributes of the struct
 * ====================================== */
impl<T, V> JkGrid<T, V>
where
    T: JkVector,
{
    /// Get the width of the grid
    #[must_use]
    pub fn width(&self) -> u32 {
        u32::value_from(self.data.shape()[0]).expect("Initialization uses u32")
    }
    /// Get the height of the grid
    #[must_use]
    pub fn height(&self) -> u32 {
        u32::value_from(self.data.shape()[1]).expect("Initialization uses u32")
    }
    /// Get the total size of the grid
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.data.len()
    }
    /// Get the data as a slice
    #[must_use]
    pub fn data_slice(&self) -> &[V] {
        self.data
            .as_slice()
            .expect("TODO: In what case is a slice non-standard?")
    }
    /// Get the data as an ndarray
    #[must_use]
    pub fn data(&self) -> &ndarray::Array2<V> {
        &self.data
    }
}

/// Defines when the user has simply exceeded the bounds of the convolution
#[derive(Debug, Clone, Error)]
#[error("({j}, k: {k}) went outside the constraints of grid")]
pub struct GridOutOfBoundsError {
    /// See [`JkVector::j`]
    j: u32,
    /// See [`JkVector::k`]
    k: u32,
}

impl JkVector for GridOutOfBoundsError {
    fn new(j: u32, k: u32) -> Self {
        Self { j, k }
    }
    fn j(&self) -> u32 {
        self.j
    }
    fn k(&self) -> u32 {
        self.k
    }
}

/* ======================================
 * Position Based Getters
 * Access data at a position
 * ====================================== */
/// Access data using JK coordinates, which are height and width respectively
impl<T, V> JkGrid<T, V>
where
    T: JkVector,
{
    /// Gets the value at the given coordinate
    ///
    /// # Panics
    /// - if idx is out of index
    ///
    /// TODO: More to convention, use square brackets
    /// TODO: Get should return Option
    #[must_use]
    pub fn get(&self, idx: T) -> &V {
        let idx = self.transform_jk_coord_to_ndarray(idx);
        &self.data[idx]
    }
    /// Gets the value at the given coordinate, or returns an error if the coordinate is out of bounds
    ///
    /// TODO: Rename to get, and change to option
    pub fn checked_get(&self, idx: T) -> Result<&V, GridOutOfBoundsError> {
        if idx.k() >= self.width() || idx.j() >= self.height() {
            return Err(GridOutOfBoundsError {
                j: idx.j(),
                k: idx.k(),
            });
        }
        Ok(self.get(idx))
    }
    /// Gets the value at the given coordinate, mutably
    pub fn get_mut(&mut self, idx: T) -> &mut V {
        let idx = self.transform_jk_coord_to_ndarray(idx);
        &mut self.data[idx]
    }
    /// Sets the value at the given coordinate, overwriting the old value
    pub fn set(&mut self, idx: T, value: V) {
        self.replace(idx, value);
    }
    /// Like set, but gives you ownership of the original value
    pub fn replace(&mut self, idx: T, replacement: V) -> V {
        let coord = self.transform_jk_coord_to_ndarray(idx);
        std::mem::replace(&mut self.data[coord], replacement)
    }
    /// Transforms the coordinate to the ndarray coordinate system using this grid's width and height
    fn transform_jk_coord_to_ndarray(&self, idx: T) -> [usize; 2] {
        [
            self.width() as usize - 1 - idx.k() as usize,
            self.height() as usize - 1 - idx.j() as usize,
        ]
    }
}

/// Iteration
impl<T, V> JkGrid<T, V>
where
    T: JkVector,
{
    /// Get an iterator over the grid
    pub fn iter(&self) -> std::slice::Iter<V> {
        self.data
            .as_slice()
            .expect("TODO: In what cases is a slice non-standard?")
            .iter()
    }

    /// Get a mutable iterator over the grid
    pub fn iter_mut(&mut self) -> std::slice::IterMut<V> {
        self.data
            .as_slice_mut()
            .expect("TODO: In what cases is a slice non-standard?")
            .iter_mut()
    }
}

impl<'a, T, V> IntoIterator for &'a JkGrid<T, V>
where
    T: JkVector,
{
    type Item = &'a V;
    type IntoIter = std::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, V> IntoIterator for &'a mut JkGrid<T, V>
where
    T: JkVector,
{
    type Item = &'a mut V;
    type IntoIter = std::slice::IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Where filter is true, get the textures
#[must_use]
pub fn filter_vecgrid<T, V>(grid: &[JkGrid<T, V>], filter: &[JkGrid<T, bool>]) -> Vec<JkGrid<T, V>>
where
    T: JkVector + Default,
    V: Default + Clone,
{
    let mut out = Vec::new();
    for (i, item) in filter.iter().enumerate() {
        let j_size = item.height();
        let k_size = item.width();
        let mut layer = JkGrid::<T, V>::new_empty(k_size, j_size);
        for j in 0..j_size {
            for k in 0..k_size {
                // Doesn't actually matter what impl JkVector type you use for this
                if *item.get(T::new(j, k)) {
                    layer.set(T::new(j, k), grid[i].get(T::new(j, k)).clone());
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
    use crate::common::util::vectors::ChunkJkVector;

    use super::*;

    #[test]
    fn test_iter() {
        let grid =
            JkGrid::<ChunkJkVector, usize>::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]).unwrap();
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
        let mut grid =
            JkGrid::<ChunkJkVector, usize>::new_from_vec(2, 3, vec![1, 2, 3, 4, 5, 6]).unwrap();

        for val in &mut grid {
            *val *= 2;
        }

        assert_eq!(grid.data_slice(), &[2, 4, 6, 8, 10, 12]);
    }
}
