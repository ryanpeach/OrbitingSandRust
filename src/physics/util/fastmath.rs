//! Fast utilities for common operations.

use std::vec::Vec;

/// A trait for fast element access in a `Vec<T>`.
///
/// Provides methods to access elements by index, returning either a reference
/// or a copy (for types that implement `Copy`).
pub trait FastVecGet<T> {
    /// Returns a reference to the element at the specified index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_ref(&self, idx: u32) -> &T;

    /// Returns a mutable reference to the element at the specified index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_mut_ref(&mut self, idx: u32) -> &mut T;

    /// Returns a copy of the element at the specified index.
    ///
    /// This method requires that `T` implements the `Copy` trait.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_copy(&self, idx: u32) -> T
    where
        T: Copy;
}

/// Implementation of the [`FastVecGet`] trait for `Vec<T>`.
///
/// Provides fast access to elements in the vector with conditional bounds checking.
/// In debug mode, bounds checking is enforced to ensure safety.
/// In release mode, bounds checking is skipped to enhance performance.
///
/// # Type Parameters
///
/// - `T`: The type of elements stored in the vector.
impl<T> FastVecGet<T> for Vec<T> {
    /// Returns a reference to the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `&self[idx]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked(idx)` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastVecGet;
    ///
    /// let vec = vec![10, 20, 30];
    /// let value_ref = vec.fast_get_ref(1);
    /// assert_eq!(*value_ref, 20);
    ///
    /// let value_copy = vec.fast_get_copy(2);
    /// assert_eq!(value_copy, 30);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_ref(&self, idx: u32) -> &T {
        #[cfg(debug_assertions)]
        {
            &self[idx as usize]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `idx` is valid.
            unsafe { self.get_unchecked(idx as usize) }
        }
    }

    /// Returns a mutable reference to the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `&self[idx]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked(idx)` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastVecGet;
    ///
    /// let vec = vec![10, 20, 30];
    /// let value_ref = vec.fast_get_ref(1);
    /// assert_eq!(*value_ref, 20);
    ///
    /// let value_copy = vec.fast_get_copy(2);
    /// assert_eq!(value_copy, 30);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_mut_ref(&mut self, idx: u32) -> &mut T {
        #[cfg(debug_assertions)]
        {
            &mut self[idx as usize]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `idx` is valid.
            unsafe { self.get_unchecked_mut(idx as usize) }
        }
    }

    /// Returns a copy of the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `self[idx]`, which performs bounds checking and copies the value.
    /// - **Release Mode**: Uses `self.get_unchecked(idx)` within an `unsafe` block,
    ///   skipping bounds checking and copying the value.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastVecGet;
    ///
    /// let vec = vec![10, 20, 30];
    /// let value = vec.fast_get_copy(1);
    /// assert_eq!(value, 20);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_copy(&self, idx: u32) -> T
    where
        T: Copy,
    {
        #[cfg(debug_assertions)]
        {
            self[idx as usize]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `idx` is valid.
            unsafe { *self.get_unchecked(idx as usize) }
        }
    }
}

/// A trait for fast element access in a 2D `Array2<T>`.
///
/// Provides methods to access elements by index, returning either a reference,
/// a mutable reference, or a copy (for types that implement [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)).
pub trait FastArray2Get<T> {
    /// Returns a reference to the element at the specified `[row, column]` index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_ref(&self, idx: [u32; 2]) -> &T;

    /// Returns a mutable reference to the element at the specified `[row, column]` index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_mut_ref(&mut self, idx: [u32; 2]) -> &mut T;

    /// Returns a copy of the element at the specified `[row, column]` index.
    ///
    /// This method requires that `T` implements the [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) trait.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_copy(&self, idx: [u32; 2]) -> T
    where
        T: Copy;
}

use ndarray::Array2;

/// Implementation of the [`FastArray2Get`] trait for [`ndarray::Array2<T>`](https://docs.rs/ndarray/latest/ndarray/struct.Array2.html).
///
/// Provides fast access to elements in the 2D array with conditional bounds checking.
/// In debug mode, bounds checking is enforced to ensure safety.
/// In release mode, bounds checking is skipped to enhance performance.
///
/// # Type Parameters
///
/// - `T`: The type of elements stored in the 2D array.
impl<T> FastArray2Get<T> for Array2<T> {
    /// Returns a reference to the element at the specified `[row, column]` index.
    ///
    /// - **Debug Mode**: Uses `&self[[row, column]]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked([row, column])` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `[row, column]` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `[row, column]` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ndarray::array;
    /// use orbiting_sand::physics::util::fastmath::FastArray2Get;
    ///
    /// let array = array![[1, 2, 3], [4, 5, 6]];
    /// let value_ref = array.fast_get_ref([0, 1]);
    /// assert_eq!(*value_ref, 2);
    ///
    /// let value_copy = array.fast_get_copy([1, 2]);
    /// assert_eq!(value_copy, 6);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_ref(&self, idx: [u32; 2]) -> &T {
        let row = idx[0] as usize;
        let col = idx[1] as usize;
        #[cfg(debug_assertions)]
        {
            &self[[row, col]]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `[row, col]` is valid.
            unsafe { self.get_unchecked([row, col]) }
        }
    }

    /// Returns a mutable reference to the element at the specified `[row, column]` index.
    ///
    /// - **Debug Mode**: Uses `&mut self[[row, column]]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked_mut([row, column])` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `[row, column]` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `[row, column]` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ndarray::array;
    /// use orbiting_sand::physics::util::fastmath::FastArray2Get;
    ///
    /// let mut array = array![[1, 2, 3], [4, 5, 6]];
    /// let value_mut_ref = array.fast_get_mut_ref([1, 0]);
    /// *value_mut_ref = 40;
    /// assert_eq!(array[[1, 0]], 40);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_mut_ref(&mut self, idx: [u32; 2]) -> &mut T {
        let row = idx[0] as usize;
        let col = idx[1] as usize;
        #[cfg(debug_assertions)]
        {
            &mut self[[row, col]]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `[row, col]` is valid.
            unsafe { self.get_unchecked_mut([row, col]) }
        }
    }

    /// Returns a copy of the element at the specified `[row, column]` index.
    ///
    /// - **Debug Mode**: Uses `self[[row, column]]`, which performs bounds checking and copies the value.
    /// - **Release Mode**: Uses `self.get_unchecked([row, column])` within an `unsafe` block,
    ///   skipping bounds checking and copying the value.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `[row, column]` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `[row, column]` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ndarray::array;
    /// use orbiting_sand::physics::util::fastmath::FastArray2Get;
    ///
    /// let array = array![[1, 2, 3], [4, 5, 6]];
    /// let value = array.fast_get_copy([0, 2]);
    /// assert_eq!(value, 3);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_copy(&self, idx: [u32; 2]) -> T
    where
        T: Copy,
    {
        let row = idx[0] as usize;
        let col = idx[1] as usize;
        #[cfg(debug_assertions)]
        {
            self[[row, col]]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `[row, col]` is valid.
            unsafe { *self.get_unchecked([row, col]) }
        }
    }
}

/// A trait for fast element access in a fixed-size array `[T; N]`.
///
/// Provides methods to access elements by index, returning either a reference,
/// a mutable reference, or a copy (for types that implement [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)).
///
/// This trait leverages Rust's conditional compilation to switch between safe and unchecked
/// access based on the build mode:
///
/// - **Debug Mode (`debug_assertions` enabled)**: Uses safe access methods that perform bounds checking.
/// - **Release Mode (`debug_assertions` disabled)**: Uses unchecked access methods to eliminate bounds checking overhead, enhancing performance.
pub trait FastArrayGet<T, const N: usize> {
    /// Returns a reference to the element at the specified index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_ref(&self, idx: u32) -> &T;

    /// Returns a mutable reference to the element at the specified index.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_mut_ref(&mut self, idx: u32) -> &mut T;

    /// Returns a copy of the element at the specified index.
    ///
    /// This method requires that `T` implements the [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) trait.
    ///
    /// In debug mode, bounds checking is performed.
    /// In release mode, bounds checking is skipped for performance.
    ///
    /// # Panics
    ///
    /// In debug mode, panics if the index is out of bounds.
    fn fast_get_copy(&self, idx: u32) -> T
    where
        T: Copy;
}

/// Implementation of the [`FastArrayGet`] trait for fixed-size arrays `[T; N]`.
///
/// Provides fast access to elements in the array with conditional bounds checking.
/// In debug mode, bounds checking is enforced to ensure safety.
/// In release mode, bounds checking is skipped to enhance performance.
///
/// # Type Parameters
///
/// - `T`: The type of elements stored in the array.
/// - `N`: The size of the array.
impl<T, const N: usize> FastArrayGet<T, N> for [T; N] {
    /// Returns a reference to the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `&self[idx]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked(idx)` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastArrayGet;
    ///
    /// let array = [10, 20, 30, 40, 50, 60, 70, 80, 90];
    /// let value_ref = array.fast_get_ref(3);
    /// assert_eq!(*value_ref, 40);
    ///
    /// let value_copy = array.fast_get_copy(5);
    /// assert_eq!(value_copy, 60);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_ref(&self, idx: u32) -> &T {
        let index = idx as usize;
        #[cfg(debug_assertions)]
        {
            &self[index]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `index` is valid.
            unsafe { self.get_unchecked(index) }
        }
    }

    /// Returns a mutable reference to the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `&mut self[idx]`, which performs bounds checking.
    /// - **Release Mode**: Uses `self.get_unchecked_mut(idx)` within an `unsafe` block,
    ///   skipping bounds checking for performance.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastArrayGet;
    ///
    /// let mut array = [10, 20, 30, 40, 50, 60, 70, 80, 90];
    /// let value_mut_ref = array.fast_get_mut_ref(2);
    /// *value_mut_ref = 35;
    /// assert_eq!(array[2], 35);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_mut_ref(&mut self, idx: u32) -> &mut T {
        let index = idx as usize;
        #[cfg(debug_assertions)]
        {
            &mut self[index]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `index` is valid.
            unsafe { self.get_unchecked_mut(index) }
        }
    }

    /// Returns a copy of the element at the specified index.
    ///
    /// - **Debug Mode**: Uses `self[idx]`, which performs bounds checking and copies the value.
    /// - **Release Mode**: Uses `self.get_unchecked(idx)` within an `unsafe` block,
    ///   skipping bounds checking and copying the value.
    ///
    /// # Panics
    ///
    /// - **Debug Mode**: Panics if `idx` is out of bounds.
    ///
    /// # Safety
    ///
    /// - **Release Mode**: It is the caller's responsibility to ensure that `idx` is within bounds.
    ///   Failing to do so results in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orbiting_sand::physics::util::fastmath::FastArrayGet;
    ///
    /// let array = [10, 20, 30, 40, 50, 60, 70, 80, 90];
    /// let value = array.fast_get_copy(7);
    /// assert_eq!(value, 80);
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    #[inline]
    fn fast_get_copy(&self, idx: u32) -> T
    where
        T: Copy,
    {
        let index = idx as usize;
        #[cfg(debug_assertions)]
        {
            self[index]
        }
        #[cfg(not(debug_assertions))]
        {
            // Safety: In release mode, the caller must ensure `index` is valid.
            unsafe { *self.get_unchecked(index) }
        }
    }
}
