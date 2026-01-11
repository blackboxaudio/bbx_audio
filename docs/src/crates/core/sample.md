# Sample Trait

The `Sample` trait abstracts over audio sample types (`f32`, `f64`), allowing DSP blocks and graphs to be generic over sample precision.

> **Note:** The `Sample` trait is defined in `bbx_core` and re-exported by `bbx_dsp` for convenience.

## Trait Definition

```rust
pub trait Sample:
    Debug
    + Copy
    + Clone
    + Send
    + Sync
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + PartialOrd
    + PartialEq
    + 'static
{
    /// Zero value (silence)
    const ZERO: Self;

    /// One value (full scale)
    const ONE: Self;

    /// Machine epsilon
    const EPSILON: Self;

    /// Mathematical constants for DSP
    const PI: Self;        // π
    const INV_PI: Self;    // 1/π
    const FRAC_PI_2: Self; // π/2
    const FRAC_PI_3: Self; // π/3
    const FRAC_PI_4: Self; // π/4
    const TAU: Self;       // 2π (full circle)
    const INV_TAU: Self;   // 1/(2π)
    const PHI: Self;       // Golden ratio
    const E: Self;         // Euler's number
    const SQRT_2: Self;    // √2
    const INV_SQRT_2: Self; // 1/√2

    /// Convert from f64
    fn from_f64(value: f64) -> Self;

    /// Convert to f64
    fn to_f64(self) -> f64;
}
```

## Implementations

The trait is implemented for `f32` and `f64`:

```rust
impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f32::EPSILON;
    const PI: Self = std::f32::consts::PI;
    const TAU: Self = std::f32::consts::TAU;
    // ... other constants with f32 precision

    fn from_f64(value: f64) -> Self { value as f32 }
    fn to_f64(self) -> f64 { self as f64 }
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f64::EPSILON;
    const PI: Self = std::f64::consts::PI;
    const TAU: Self = std::f64::consts::TAU;
    // ... other constants with f64 precision

    fn from_f64(value: f64) -> Self { value }
    fn to_f64(self) -> f64 { self }
}
```

## Mathematical Constants

The `Sample` trait provides mathematical constants commonly used in DSP:

| Constant | Value | Common DSP Use |
|----------|-------|----------------|
| `PI` | π ≈ 3.14159 | Phase calculations, filter coefficients |
| `TAU` | 2π ≈ 6.28318 | Full cycle/phase wrap, angular frequency |
| `INV_TAU` | 1/(2π) | Frequency-to-phase conversion |
| `FRAC_PI_2` | π/2 | Quarter-wave, phase shifts |
| `SQRT_2` | √2 ≈ 1.414 | RMS calculations, equal-power panning |
| `INV_SQRT_2` | 1/√2 ≈ 0.707 | Equal-power crossfade, normalization |
| `E` | e ≈ 2.718 | Exponential decay, RC filter time constants |
| `PHI` | φ ≈ 1.618 | Golden ratio for aesthetic frequency ratios |

These constants are provided at compile-time precision for both `f32` and `f64`, avoiding runtime conversions in hot paths.

## Usage

### Generic DSP Code

Write DSP code that works with any sample type:

```rust
use bbx_core::sample::Sample;

fn apply_gain<S: Sample>(samples: &mut [S], gain_db: f64) {
    let linear = S::from_f64((10.0_f64).powf(gain_db / 20.0));
    for sample in samples {
        *sample = *sample * linear;
    }
}

// Works with both f32 and f64
let mut samples_f32: Vec<f32> = vec![0.5, 0.3, 0.1];
let mut samples_f64: Vec<f64> = vec![0.5, 0.3, 0.1];

apply_gain(&mut samples_f32, -6.0);
apply_gain(&mut samples_f64, -6.0);
```

### Using Constants

```rust
use bbx_core::sample::Sample;

fn normalize<S: Sample>(samples: &mut [S]) {
    let max = samples.iter()
        .map(|s| if *s < S::ZERO { S::ZERO - *s } else { *s })
        .fold(S::ZERO, |a, b| if a > b { a } else { b });

    if max > S::ZERO {
        for sample in samples {
            *sample = *sample / max;
        }
    }
}
```

## SIMD Support

When the `simd` feature is enabled, the `Sample` trait provides additional associated types and methods for vectorized processing.

### Enabling SIMD

```toml
[dependencies]
bbx_core = { version = "...", features = ["simd"] }
```

### SIMD Associated Type and Methods

With the `simd` feature, the trait includes:

```rust
pub trait Sample {
    // ... base methods ...

    /// The SIMD vector type (f32x4 for f32, f64x4 for f64)
    #[cfg(feature = "simd")]
    type Simd: SimdFloat<Scalar = Self> + StdFloat + SimdPartialOrd + Copy + ...;

    /// Create a SIMD vector with all lanes set to the given value
    #[cfg(feature = "simd")]
    fn simd_splat(value: Self) -> Self::Simd;

    /// Load a SIMD vector from a slice (must have at least 4 elements)
    #[cfg(feature = "simd")]
    fn simd_from_slice(slice: &[Self]) -> Self::Simd;

    /// Convert a SIMD vector to an array
    #[cfg(feature = "simd")]
    fn simd_to_array(simd: Self::Simd) -> [Self; 4];

    /// Select elements where a > b
    #[cfg(feature = "simd")]
    fn simd_select_gt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd;

    /// Select elements where a < b
    #[cfg(feature = "simd")]
    fn simd_select_lt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd;

    /// Returns lane offsets [0.0, 1.0, 2.0, 3.0] for phase calculations
    #[cfg(feature = "simd")]
    fn simd_lane_offsets() -> Self::Simd;
}
```

### Generic SIMD Example

Write SIMD-accelerated code that works for both `f32` and `f64`:

```rust
use bbx_core::sample::{Sample, SIMD_LANES};

#[cfg(feature = "simd")]
fn apply_gain_simd<S: Sample>(output: &mut [S], gain: S) {
    let gain_vec = S::simd_splat(gain);
    let (chunks, remainder) = output.as_chunks_mut::<SIMD_LANES>();

    for chunk in chunks {
        let samples = S::simd_from_slice(chunk);
        let result = samples * gain_vec;
        chunk.copy_from_slice(&S::simd_to_array(result));
    }

    for sample in remainder {
        *sample = *sample * gain;
    }
}
```

## Choosing a Sample Type

### f32 (Recommended)

- Smaller memory footprint (4 bytes vs 8 bytes)
- Better SIMD throughput
- Sufficient precision for most audio work
- Standard for most audio APIs

### f64

- Higher precision for:
  - Long delay lines
  - Accumulating filters
  - Scientific/measurement applications
- Common in offline processing
