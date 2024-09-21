use crate::sample::Sample;

pub struct Buffer<S: Sample> {
    capacity: usize,
    data: Vec<S>,
}

impl<S: Sample> Buffer<S> {
    pub fn new(capacity: usize) -> Self {
        Buffer {
            capacity,
            data: vec![S::EQUILIBRIUM; capacity],
        }
    }

    pub fn from_slice(slice: &[S]) -> Self {
        Buffer {
            capacity: slice.len(),
            data: slice.to_vec(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn apply<F: Fn(S) -> S>(&mut self, f: F) {
        self.data = self.data.iter().map(|&s| f(s)).collect();
    }

    pub fn equilibrium(&mut self) {
        self.data = vec![S::EQUILIBRIUM; self.capacity];
    }

    pub fn get_sample(&self, idx: usize) -> Option<&S> {
        self.data.get(idx)
    }

    pub fn set_sample(&mut self, idx: usize, value: S) {
        if idx < self.data.len() {
            self.data[idx] = value;
        }
    }
}
