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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_range() {
        let mut rng = XorShiftRng::new(12345);
        for _ in 0..10000 {
            let sample = rng.next_noise_sample();
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "Sample {} out of [-1, 1] range",
                sample
            );
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let mut rng1 = XorShiftRng::new(42);
        let mut rng2 = XorShiftRng::new(42);

        for _ in 0..100 {
            let s1 = rng1.next_noise_sample();
            let s2 = rng2.next_noise_sample();
            assert!((s1 - s2).abs() < 1e-15, "Same seed should produce identical sequences");
        }
    }

    #[test]
    fn test_different_seeds_produce_different_sequences() {
        let mut rng1 = XorShiftRng::new(1);
        let mut rng2 = XorShiftRng::new(2);

        let mut all_same = true;
        for _ in 0..100 {
            let s1 = rng1.next_noise_sample();
            let s2 = rng2.next_noise_sample();
            if (s1 - s2).abs() > 1e-15 {
                all_same = false;
                break;
            }
        }
        assert!(!all_same, "Different seeds should produce different sequences");
    }

    #[test]
    fn test_zero_seed_handled() {
        let mut rng = XorShiftRng::new(0);
        let sample = rng.next_noise_sample();
        assert!(
            sample >= -1.0 && sample <= 1.0,
            "Zero seed should work (converted to 1)"
        );
    }

    #[test]
    fn test_default_produces_valid_output() {
        let mut rng = XorShiftRng::default();
        for _ in 0..100 {
            let sample = rng.next_noise_sample();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_not_constant() {
        let mut rng = XorShiftRng::new(42);
        let first = rng.next_noise_sample();
        let mut found_different = false;

        for _ in 0..100 {
            let sample = rng.next_noise_sample();
            if (sample - first).abs() > 1e-10 {
                found_different = true;
                break;
            }
        }
        assert!(found_different, "RNG should produce varying values");
    }

    #[test]
    fn test_distribution_rough_balance() {
        let mut rng = XorShiftRng::new(54321);
        let mut positive_count = 0;
        let mut negative_count = 0;
        let num_samples = 10000;

        for _ in 0..num_samples {
            let sample = rng.next_noise_sample();
            if sample > 0.0 {
                positive_count += 1;
            } else if sample < 0.0 {
                negative_count += 1;
            }
        }

        let ratio = positive_count as f64 / negative_count as f64;

        // With 10k samples, a perfect uniform distribution would have ~5000 each.
        // Bounds of [0.8, 1.25] allow for ±20% deviation, which is ~4σ for n=10000 (very loose).
        // This validates basic uniformity without being flaky due to statistical variance.
        assert!(
            ratio > 0.8 && ratio < 1.25,
            "Distribution should be roughly balanced: pos={}, neg={}, ratio={}",
            positive_count,
            negative_count,
            ratio
        );
    }
}
