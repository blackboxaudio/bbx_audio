# Testing

Testing strategies for bbx_audio.

## Running Tests

```bash
# All tests
cargo test --workspace --release

# Specific crate
cargo test -p bbx_dsp --release

# Specific test
cargo test test_oscillator --release
```

## Test Categories

### Unit Tests

In-module tests for individual components:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_calculation() {
        let block = GainBlock::<f32>::new(-6.0);
        let expected = 0.5; // -6 dB ≈ 0.5
        assert!((block.multiplier() - expected).abs() < 0.01);
    }
}
```

### Integration Tests

Cross-module tests in `tests/` directory:

```rust
// tests/graph_tests.rs
use bbx_dsp::{GraphBuilder, Waveform};

#[test]
fn test_simple_graph() {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let graph = builder.build();

    // Test processing...
}
```

## Audio Testing Challenges

Audio tests are inherently approximate:

```rust
#[test]
fn test_sine_output() {
    // Generate one cycle of 1 Hz at 4 samples/second
    // Expected: [0, 1, 0, -1]

    let output = generate_sine(4);

    // Use epsilon comparison
    assert!((output[0] - 0.0).abs() < 0.001);
    assert!((output[1] - 1.0).abs() < 0.001);
}
```

## Test Utilities

### DspContext for Tests

```rust
fn test_context() -> DspContext {
    DspContext::new(44100.0, 512, 2)
}
```

### Buffer Helpers

```rust
fn create_test_buffers(size: usize) -> (Vec<f32>, Vec<f32>) {
    (vec![0.0; size], vec![0.0; size])
}
```

## What to Test

1. **Edge cases** - Zero input, maximum values
2. **Parameter ranges** - Valid and invalid values
3. **Sample types** - Both f32 and f64
4. **Processing correctness** - Expected output values
5. **State management** - Reset behavior
