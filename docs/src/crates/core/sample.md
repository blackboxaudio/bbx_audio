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

    fn from_f64(value: f64) -> Self { value as f32 }
    fn to_f64(self) -> f64 { self as f64 }
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn from_f64(value: f64) -> Self { value }
    fn to_f64(self) -> f64 { self }
}
```

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
