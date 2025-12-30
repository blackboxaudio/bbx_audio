/// Used for generating pseudo-random numbers
/// quickly.
pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    /// Create an instance of the `XorShiftRng` pseudo-random
    /// number generator with a specific seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    /// Generate the next pseudo-random number.
    pub fn next_noise_sample(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;

        ((self.state as f64) / (u64::MAX as f64)) * 2.0 - 1.0
    }
}

impl Default for XorShiftRng {
    /// Create a default instance of the `XorShiftRng`
    /// pseudo-random number generator.
    fn default() -> Self {
        Self { state: 1 }
    }
}
