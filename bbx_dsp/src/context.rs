pub struct Context {
    pub sample_rate: usize,
    pub buffer_size: usize,
}

impl Context {
    pub fn new(sample_rate: usize, buffer_size: usize) -> Self {
        Context { sample_rate, buffer_size }
    }
}
