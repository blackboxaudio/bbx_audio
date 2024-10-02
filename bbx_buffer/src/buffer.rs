use std::ops::{Index, IndexMut};

use bbx_sample::sample::Sample;

/// An iterable container for data, usually audio samples or
/// modulation data.
#[allow(clippy::len_without_is_empty)]
pub trait Buffer<T> {
    /// Creates a new `Buffer` with some capacity.
    fn new(capacity: usize) -> Self;

    /// Creates a new `Buffer` from a referenced slice of data.
    fn from_slice(slice: &[T]) -> Self;

    /// Returns a `Buffer` represented as a slice.
    fn as_slice(&self) -> &[T];

    /// Copies data from an existing `Buffer` into this one.
    fn copy_from_slice(&mut self, slice: &[T]);

    /// Returns the length of the `Buffer`.
    fn len(&self) -> usize;

    /// Applies a function to the data inside the `Buffer`.
    fn apply<F: Fn(T) -> T>(&mut self, f: F);

    /// Applies a mutable closure to the data inside the `Buffer`.
    fn apply_mut<F: FnMut(T) -> T>(&mut self, f: F);

    /// Zeroes the data within the `Buffer`, retaining its original size
    /// to avoid dynamic allocation.
    fn clear(&mut self);
}

/// A `Buffer` specifically for data that is trait-bound to
/// a `Sample`.
#[derive(Clone)]
pub struct AudioBuffer<S: Sample> {
    capacity: usize,
    next_idx: usize,
    data: Vec<S>,
}

impl<S: Sample> Buffer<S> for AudioBuffer<S> {
    fn new(capacity: usize) -> Self {
        AudioBuffer {
            capacity,
            next_idx: 0,
            data: vec![S::EQUILIBRIUM; capacity],
        }
    }

    fn from_slice(slice: &[S]) -> Self {
        AudioBuffer {
            capacity: slice.len(),
            next_idx: 0,
            data: slice.to_vec(),
        }
    }

    fn as_slice(&self) -> &[S] {
        self.data.as_slice()
    }

    fn copy_from_slice(&mut self, slice: &[S]) {
        self.data.copy_from_slice(slice);
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn apply<F: Fn(S) -> S>(&mut self, f: F) {
        self.data = self.data.iter().map(|&s| f(s)).collect()
    }

    fn apply_mut<F: FnMut(S) -> S>(&mut self, mut f: F) {
        self.data = self.data.iter().map(|&s| f(s)).collect()
    }

    fn clear(&mut self) {
        self.apply(|_s| S::EQUILIBRIUM);
    }
}

impl<S: Sample> Iterator for AudioBuffer<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx < self.capacity {
            self.next_idx += 1;
            Some(self.data[self.next_idx - 1])
        } else {
            None
        }
    }
}

impl<S: Sample> Index<usize> for AudioBuffer<S> {
    type Output = S;

    fn index(&self, idx: usize) -> &Self::Output {
        self.data.get(idx).unwrap()
    }
}

impl<S: Sample> IndexMut<usize> for AudioBuffer<S> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.data.get_mut(idx).unwrap()
    }
}
