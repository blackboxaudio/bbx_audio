# SIMD Optimizations

SIMD (Single Instruction Multiple Data) support for accelerated DSP processing.

## Enabling SIMD

Enable the `simd` feature flag in your `Cargo.toml`:

```toml
[dependencies]
bbx_dsp = { version = "...", features = ["simd"] }
```

**Requirements:**
- Nightly Rust toolchain (uses the unstable `portable_simd` feature)
- Build with: `cargo +nightly build --features simd`

## How It Works

SIMD processes multiple samples simultaneously:

```
Scalar: a[0]*b[0], a[1]*b[1], a[2]*b[2], a[3]*b[3]  (4 operations)
SIMD:   a[0:3] * b[0:3]                              (1 operation)
```

The implementation uses 4-lane vectors (`f32x4` and `f64x4`) from Rust's `std::simd`.

## SIMD Operations

The `bbx_core::simd` module provides these vectorized operations:

| Function | Description |
|----------|-------------|
| `fill_f32/f64` | Fill a buffer with a constant value |
| `apply_gain_f32/f64` | Multiply samples by a gain factor |
| `multiply_add_f32/f64` | Element-wise multiplication of two buffers |
| `sin_f32/f64` | Vectorized sine computation |

Additionally, the `denormal` module provides SIMD-accelerated batch denormal flushing:
- `flush_denormals_f32_batch`
- `flush_denormals_f64_batch`

## Optimized Blocks

The following blocks use SIMD when the feature is enabled:

| Block | Optimization |
|-------|--------------|
| `OscillatorBlock` | Vectorized waveform generation (4 samples at a time) |
| `LfoBlock` | Vectorized modulation signal generation |
| `GainBlock` | Vectorized gain application |
| `PannerBlock` | Vectorized sin/cos gain calculation |

## Feature Propagation

The `simd` feature propagates through crate dependencies:

```
bbx_plugin --simd--> bbx_dsp --simd--> bbx_core
```

Enable `simd` on `bbx_plugin` for plugin builds:

```toml
[dependencies]
bbx_plugin = { version = "...", features = ["simd"] }
```

## Trade-offs

| Aspect | Scalar | SIMD |
|--------|--------|------|
| Complexity | Simple | More complex |
| Toolchain | Stable Rust | Nightly required |
| Debugging | Easy | Harder |
| Performance | Baseline | Up to 4x faster |

## Implementation Notes

- Lane width is 4 for both `f32` and `f64` (SSE/NEON compatible)
- Remainder samples (when buffer size isn't divisible by 4) are processed with scalar fallback
- Noise waveforms use scalar processing due to RNG sequentiality requirements
