use crate::sample::Sample;

pub trait Buffer<T> {
    fn new(capacity: usize) -> Self;
    fn from_slice(slice: &[T]) -> Self;

    fn len(&self) -> usize;

    fn apply<F: Fn(T) -> T>(&mut self, f: F);

    fn clear(&mut self);
}

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
