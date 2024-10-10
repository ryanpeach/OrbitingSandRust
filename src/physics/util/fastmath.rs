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
    /// use crate::FastVecGet;
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
    /// use crate::FastVecGet;
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
