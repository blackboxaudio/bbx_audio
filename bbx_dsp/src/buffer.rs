use crate::sample::Sample;

/// An iterable container for data, usually audio samples or
/// modulation data.
pub trait Buffer<T> {
    /// Creates a new `Buffer` with some capacity.
    fn new(capacity: usize) -> Self;

    /// Creates a new `Buffer` from a referenced slice of data.
    fn from_slice(slice: &[T]) -> Self;

    /// Returns the length of the `Buffer`.
    fn len(&self) -> usize;

    /// Applies a function to the data inside the `Buffer`.
    fn apply<F: Fn(T) -> T>(&mut self, f: F);

    /// Zeroes the data within the `Buffer`, retaining its original size
    /// to avoid dynamic allocation.
    fn clear(&mut self);
}

/// A `Buffer` specifically for data that is trait-bound to
/// a `Sample`.
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

    fn len(&self) -> usize {
        self.data.len()
    }

    fn apply<F: Fn(S) -> S>(&mut self, f: F) {
        self.data = self.data.iter().map(|&s| f(s)).collect()
    }

    fn clear(&mut self) {
        self.data = vec![S::EQUILIBRIUM; self.capacity];
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
