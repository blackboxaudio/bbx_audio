# Sample Trait

The `Sample` trait abstracts over audio sample types (f32, f64).

## Trait Definition

```rust
pub trait Sample:
    Copy
    + Clone
    + Default
    + Send
    + Sync
    + PartialOrd
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::Neg<Output = Self>
    + 'static
{
    /// Zero value
    const ZERO: Self;

    /// One value
    const ONE: Self;

    /// Minimum representable value
    const MIN: Self;

    /// Maximum representable value
    const MAX: Self;

    /// Convert from f32
    fn from_f32(value: f32) -> Self;

    /// Convert from f64
    fn from_f64(value: f64) -> Self;

    /// Convert to f32
    fn to_f32(self) -> f32;

    /// Convert to f64
    fn to_f64(self) -> f64;

    /// Absolute value
    fn abs(self) -> Self;

    /// Sine function
    fn sin(self) -> Self;

    /// Cosine function
    fn cos(self) -> Self;

    /// Power function
    fn powf(self, n: Self) -> Self;

    /// Square root
    fn sqrt(self) -> Self;

    /// Floor
    fn floor(self) -> Self;

    /// Clamp to range
    fn clamp(self, min: Self, max: Self) -> Self;
}
```

## Implementations

The trait is implemented for `f32` and `f64`:

```rust
impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MIN: Self = f32::MIN;
    const MAX: Self = f32::MAX;

    fn from_f32(value: f32) -> Self { value }
    fn from_f64(value: f64) -> Self { value as f32 }
    fn to_f32(self) -> f32 { self }
    fn to_f64(self) -> f64 { self as f64 }
    // ... etc
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    // ... etc
}
```

## Usage

### Generic DSP Code

Write DSP code that works with any sample type:

```rust
use bbx_dsp::sample::Sample;

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
use bbx_dsp::sample::Sample;

fn normalize<S: Sample>(samples: &mut [S]) {
    let max = samples.iter()
        .map(|s| s.abs())
        .fold(S::ZERO, |a, b| if a > b { a } else { b });

    if max > S::ZERO {
        for sample in samples {
            *sample = *sample / max;
        }
    }
}
```

### Converting Types

```rust
use bbx_dsp::sample::Sample;

fn to_16bit<S: Sample>(sample: S) -> i16 {
    let clamped = sample.clamp(S::from_f32(-1.0), S::from_f32(1.0));
    (clamped.to_f64() * 32767.0) as i16
}
```

## Choosing a Sample Type

### f32 (Recommended)

- Smaller memory footprint
- Faster SIMD operations
- Sufficient precision for most audio work
- Standard for most audio APIs

### f64

- Higher precision for:
  - Long delay lines
  - Accumulating filters
  - Scientific/measurement applications
- Common in offline processing
