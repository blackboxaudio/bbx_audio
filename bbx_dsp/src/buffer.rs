use std::ops::{Index, IndexMut};

use crate::sample::Sample;

/// Describes a buffer of an arbitrary data type `T`.
pub trait Buffer<T> {
    /// Get the length of the `Buffer`.
    fn len(&self) -> usize;

    /// Check if the `Buffer` is empty.
    fn is_empty(&self) -> bool;

    /// Get the `Buffer` as a data slice.
    fn as_slice(&self) -> &[T];

    /// Get the `Buffer` as a mutable data slice.
    fn as_mut_slice(&mut self) -> &mut [T];

    /// Remove all values from the `Buffer`
    fn clear(&mut self);

    /// Set all values in the `Buffer` to zero.
    fn zeroize(&mut self);
}

/// Used for containing audio sample data.
#[derive(Debug, Clone)]
pub struct AudioBuffer<S: Sample> {
    data: Vec<S>,
}

impl<S: Sample> AudioBuffer<S> {
    /// Create an `AudioBuffer` of a particular capacity,
    /// automatically filled with zeroes.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![S::ZERO; capacity],
        }
    }

    /// Create an `AudioBuffer` initialized with a given set of values.
    pub fn with_data(data: Vec<S>) -> Self {
        Self { data }
    }

    /// Get the capacity of the `AudioBuffer`.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Fill the `AudioBuffer` with a particular value.
    pub fn fill(&mut self, value: S) {
        self.data.fill(value);
    }

    /// Copy the values from a given source to the `AudioBuffer`.
    pub fn copy_from_slice(&mut self, source: &[S]) {
        self.data[..source.len()].copy_from_slice(source);
    }

    /// Copy the `AudioBuffer`'s values to a given slice.
    pub fn copy_to_slice(&self, target: &mut [S]) {
        let len = self.data.len().min(target.len());
        target[..len].copy_from_slice(&self.data[..len]);
    }

    /// Get the `AudioBuffer` as a pointer to its contents.
    pub fn as_ptr(&self) -> *const S {
        self.data.as_ptr()
    }

    /// Get the `AudioBuffer` as a mutable pointer to its contents.
    pub fn as_mut_ptr(&mut self) -> *mut S {
        self.data.as_mut_ptr()
    }

    /// Append the `AudioBuffer` with data from a given slice.
    pub fn extend_from_slice(&mut self, slice: &[S]) {
        self.data.extend_from_slice(slice);
    }

    /// Drain the first `count` values from the `AudioBuffer`.
    pub fn drain_front(&mut self, count: usize) -> std::vec::Drain<'_, S> {
        let actual_count = count.min(self.data.len());
        self.data.drain(0..actual_count)
    }
}

impl<S: Sample> Buffer<S> for AudioBuffer<S> {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn as_slice(&self) -> &[S] {
        &self.data
    }

    fn as_mut_slice(&mut self) -> &mut [S] {
        &mut self.data
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn zeroize(&mut self) {
        self.data.fill(S::ZERO);
    }
}

impl<S: Sample> Index<usize> for AudioBuffer<S> {
    type Output = S;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<S: Sample> IndexMut<usize> for AudioBuffer<S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<S: Sample> Extend<S> for AudioBuffer<S> {
    fn extend<I: IntoIterator<Item = S>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}
