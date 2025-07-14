pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_noise_sample(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;

        ((self.state as f64) / (u64::MAX as f64)) * 2.0 - 1.0
    }
}

impl Default for XorShiftRng {
    fn default() -> Self {
        Self { state: 1 }
    }
}
