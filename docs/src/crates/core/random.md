# Random Number Generation

A fast XorShift64 random number generator suitable for audio applications.

## Overview

`XorShiftRng` provides:

- Fast pseudo-random number generation
- Deterministic output (given the same seed)
- Audio-range output methods (-1.0 to 1.0)

## API

### Creating an RNG

```rust
use bbx_core::random::XorShiftRng;

// Create with a seed
let mut rng = XorShiftRng::new(42);

// Create with a different seed for different sequences
let mut rng2 = XorShiftRng::new(12345);
```

### Generating Numbers

```rust
use bbx_core::random::XorShiftRng;

let mut rng = XorShiftRng::new(42);

// Raw u64 value
let raw = rng.next_u64();

// Floating-point 0.0 to 1.0
let normalized = rng.next_f32();  // or next_f64()

// Audio sample -1.0 to 1.0
let sample = rng.next_noise_sample();
```

## Usage in Audio

### White Noise Generator

```rust
use bbx_core::random::XorShiftRng;

struct NoiseGenerator {
    rng: XorShiftRng,
}

impl NoiseGenerator {
    fn new(seed: u64) -> Self {
        Self {
            rng: XorShiftRng::new(seed),
        }
    }

    fn process(&mut self, output: &mut [f32]) {
        for sample in output {
            *sample = self.rng.next_noise_sample();
        }
    }
}
```

### Randomized Modulation

```rust
use bbx_core::random::XorShiftRng;

struct RandomLfo {
    rng: XorShiftRng,
    current_value: f32,
    target_value: f32,
    smoothing: f32,
}

impl RandomLfo {
    fn new(seed: u64, smoothing: f32) -> Self {
        let mut rng = XorShiftRng::new(seed);
        let initial = rng.next_noise_sample();
        Self {
            rng,
            current_value: initial,
            target_value: initial,
            smoothing,
        }
    }

    fn process(&mut self) -> f32 {
        // Occasionally pick a new target
        if self.rng.next_f32() < 0.001 {
            self.target_value = self.rng.next_noise_sample();
        }

        // Smooth toward target
        self.current_value += (self.target_value - self.current_value) * self.smoothing;
        self.current_value
    }
}
```

## Algorithm

XorShift64 is a simple but effective pseudo-random number generator:

```rust
fn next_u64(&mut self) -> u64 {
    let mut x = self.state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    self.state = x;
    x
}
```

Properties:
- Period: 2^64 - 1
- Fast: ~3 CPU cycles per number
- Good statistical properties for audio use
- Not cryptographically secure

## Seeding

Different seeds produce completely different sequences:

```rust
use bbx_core::random::XorShiftRng;

let mut rng1 = XorShiftRng::new(1);
let mut rng2 = XorShiftRng::new(2);

// Completely different sequences
assert_ne!(rng1.next_u64(), rng2.next_u64());
```

For reproducible results (e.g., testing), use a fixed seed.

For unique sequences each run, use a time-based seed:

```rust
use bbx_core::random::XorShiftRng;
use std::time::{SystemTime, UNIX_EPOCH};

let seed = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos() as u64;

let mut rng = XorShiftRng::new(seed);
```
