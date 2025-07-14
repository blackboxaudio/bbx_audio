use std::ops::{Index, IndexMut};

use crate::sample::Sample;

pub trait Buffer<T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
    fn clear(&mut self);
    fn zeroize(&mut self);
}

#[derive(Debug, Clone)]
pub struct AudioBuffer<S: Sample> {
    data: Vec<S>,
}

impl<S: Sample> AudioBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![S::ZERO; capacity],
        }
    }

    pub fn with_data(data: Vec<S>) -> Self {
        Self { data }
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn fill(&mut self, value: S) {
        self.data.fill(value);
    }

    pub fn copy_from_slice(&mut self, source: &[S]) {
        self.data[..source.len()].copy_from_slice(source);
    }

    pub fn copy_to_slice(&self, target: &mut [S]) {
        let len = self.data.len().min(target.len());
        target[..len].copy_from_slice(&self.data[..len]);
    }

    pub fn as_ptr(&self) -> *const S {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut S {
        self.data.as_mut_ptr()
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S>,
    {
        self.data.extend(iter);
    }

    pub fn extend_from_slice(&mut self, slice: &[S]) {
        self.data.extend_from_slice(slice);
    }

    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, S>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.data.drain(range)
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

impl<S: Sample> FromIterator<S> for AudioBuffer<S> {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Self {
            data: Vec::from_iter(iter),
        }
    }
}
