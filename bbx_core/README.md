# bbx_core

Foundational utilities and data structures for the bbx_audio workspace.

## Features

- **Denormal handling**: Flush denormal floating-point values to zero
- **Error types**: Unified error handling across crates
- **Lock-free SPSC**: Single-producer single-consumer ring buffer
- **Stack-allocated vector**: Fixed-capacity vector without heap allocation
- **RNG**: Fast XorShift random number generator

## Cargo Features

### `ftz-daz`

Enables hardware-level denormal prevention on x86/x86_64 processors. When enabled, the `enable_ftz_daz()` function becomes available:

```rust
use bbx_core::denormal::enable_ftz_daz;

// Call once at the start of each audio thread
enable_ftz_daz();
```

This sets CPU flags (FTZ and DAZ) to automatically flush denormal floats to zero, avoiding the 10-100x slowdowns they can cause. Recommended for production audio applications.

## Modules

### `denormal`

Utilities for handling denormal (subnormal) floating-point values, preventing CPU slowdowns during quiet audio passages.

```rust
use bbx_core::{flush_denormal_f32, flush_denormal_f64};

let value = flush_denormal_f64(very_small_value);
```

### `spsc`

A lock-free single-producer single-consumer ring buffer for inter-thread communication in audio applications.

```rust
use bbx_core::SpscRingBuffer;

let (producer, consumer) = SpscRingBuffer::new::<f32>(1024);

// Producer thread
producer.try_push(sample);

// Consumer thread
if let Some(sample) = consumer.try_pop() {
    // Process sample
}
```

### `stack_vec`

A fixed-capacity vector that stores elements on the stack, avoiding heap allocations in performance-critical code.

```rust
use bbx_core::StackVec;

let mut vec: StackVec<f32, 8> = StackVec::new();
vec.push(1.0);
vec.push(2.0);
```

### `random`

A fast XorShift64 random number generator suitable for audio applications (noise generation, etc.).

```rust
use bbx_core::random::XorShiftRng;

let mut rng = XorShiftRng::new(42);
let noise_sample = rng.next_noise_sample(); // Returns -1.0 to 1.0
```

### `error`

Unified error types for the workspace.

```rust
use bbx_core::{BbxError, Result};

fn process() -> Result<()> {
    // ...
    Ok(())
}
```

## License

MIT
