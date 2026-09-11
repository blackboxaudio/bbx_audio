# bbx_core

Foundational utilities and data structures for the bbx_audio workspace.

## Features

- **Denormal handling**: Flush denormal floating-point values to zero
- **Error types**: Unified error handling across crates
- **Lock-free SPSC**: Single-producer single-consumer ring buffer
- **Stack-allocated vector**: Fixed-capacity vector without heap allocation
- **RNG**: Fast XorShift random number generator
- **Lookup tables**: Compile-time `SineTable` and a shared interpolating table read

## Cargo Features

### `ftz-daz`

Enables hardware-level denormal prevention on x86/x86_64 and AArch64 (Apple Silicon) processors. When enabled, the `enable_ftz_daz()` function becomes available:

```rust
use bbx_core::denormal::enable_ftz_daz;

// Call once at the start of each audio thread
enable_ftz_daz();
```

This sets CPU flags to automatically flush denormal floats to zero, avoiding the 10-100x slowdowns they can cause. On x86/x86_64, this enables both FTZ and DAZ modes. On AArch64, only FTZ is available—use `flush_denormal_f64/f32` in feedback paths for full coverage. Recommended for production audio applications.

### `simd`

Enables SIMD-accelerated operations for common DSP tasks. Requires nightly Rust due to the unstable `portable_simd` feature.

```toml
[dependencies]
bbx_core = { version = "...", features = ["simd"] }
```

Provides the `simd` module with vectorized operations:
- `fill_f32/f64` - Fill buffers with a value
- `apply_gain_f32/f64` - Apply gain to samples
- `multiply_add_f32/f64` - Element-wise multiplication
- `sin_f32/f64` - Vectorized sine

Also enables SIMD-accelerated `flush_denormals_f32/f64_batch` functions in the `denormal` module.

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
let _ = vec.push(1.0);
let _ = vec.push(2.0);
```

### `random`

A fast XorShift64 random number generator suitable for audio applications (noise generation, etc.).

```rust
use bbx_core::random::XorShiftRng;

let mut rng = XorShiftRng::new(42);
let noise_sample = rng.next_noise_sample(); // Returns -1.0 to 1.0
```

### `table`

Lookup tables read by phase in cycles. `SineTable` is evaluated at compile time (`no_std` friendly), and `read_interpolated` is the shared linear-interpolating reader for any `[f32; LENGTH]` table laid out as `LENGTH - 1` cells plus a guard entry.

```rust
use bbx_core::{SINE_2048, SineTable};

static SMALL: SineTable<1025> = SineTable::new();

let sine = SINE_2048.read(0.25);         // sin(2π · 0.25) ≈ 1.0
let cosine = SINE_2048.read_cosine(0.0); // same table, a quarter cycle ahead
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
