//! Sample buffer types.
//!
//! This module provides [`SampleBuffer`] for storing audio sample data during DSP processing.

use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

use bbx_core::Buffer;
#[cfg(feature = "simd")]
use bbx_core::simd::fill as simd_fill;

use crate::sample::Sample;

/// A buffer for storing audio sample data.
///
/// Used internally by the DSP graph for block input/output buffers.
/// Wraps a `Vec<S>` with convenience methods for audio processing.
#[derive(Debug, Clone)]
pub struct SampleBuffer<S: Sample> {
    data: Vec<S>,
}

impl<S: Sample> SampleBuffer<S> {
    /// Create a `SampleBuffer` of a particular capacity,
    /// automatically filled with zeroes.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![S::ZERO; capacity],
        }
    }

    /// Create a `SampleBuffer` initialized with a given set of values.
    #[inline]
    pub fn with_data(data: Vec<S>) -> Self {
        Self { data }
    }

    /// Get the capacity of the `SampleBuffer`.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Fill the `SampleBuffer` with a particular value.
    #[inline]
    pub fn fill(&mut self, value: S) {
        #[cfg(feature = "simd")]
        simd_fill(self.data.as_mut_slice(), value);

        #[cfg(not(feature = "simd"))]
        self.data.fill(value);
    }

    /// Copy the values from a given source to the `SampleBuffer`.
    #[inline]
    pub fn copy_from_slice(&mut self, source: &[S]) {
        self.data[..source.len()].copy_from_slice(source);
    }

    /// Copy the `SampleBuffer`'s values to a given slice.
    #[inline]
    pub fn copy_to_slice(&self, target: &mut [S]) {
        let len = self.data.len().min(target.len());
        target[..len].copy_from_slice(&self.data[..len]);
    }

    /// Get the `SampleBuffer` as a pointer to its contents.
    #[inline]
    pub fn as_ptr(&self) -> *const S {
        self.data.as_ptr()
    }

    /// Get the `SampleBuffer` as a mutable pointer to its contents.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut S {
        self.data.as_mut_ptr()
    }

    /// Append the `SampleBuffer` with data from a given slice.
    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[S]) {
        self.data.extend_from_slice(slice);
    }

    /// Drain the first `count` values from the `SampleBuffer`.
    #[inline]
    pub fn drain_front(&mut self, count: usize) -> alloc::vec::Drain<'_, S> {
        let actual_count = count.min(self.data.len());
        self.data.drain(0..actual_count)
    }
}

impl<S: Sample> Buffer<S> for SampleBuffer<S> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    fn as_slice(&self) -> &[S] {
        &self.data
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [S] {
        &mut self.data
    }

    #[inline]
    fn clear(&mut self) {
        self.data.clear();
    }

    #[inline]
    fn zeroize(&mut self) {
        #[cfg(feature = "simd")]
        simd_fill(self.data.as_mut_slice(), S::ZERO);

        #[cfg(not(feature = "simd"))]
        self.data.fill(S::ZERO);
    }
}

impl<S: Sample> Index<usize> for SampleBuffer<S> {
    type Output = S;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<S: Sample> IndexMut<usize> for SampleBuffer<S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<S: Sample> Extend<S> for SampleBuffer<S> {
    fn extend<I: IntoIterator<Item = S>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
}
