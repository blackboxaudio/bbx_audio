//! Audio sample type abstraction.
//!
//! This module defines the [`Sample`] trait, which abstracts over floating-point
//! types used for audio processing. This allows blocks and graphs to be generic
//! over sample precision (`f32` or `f64`).

#![allow(clippy::approx_constant)]
#![allow(clippy::excessive_precision)]

#[cfg(feature = "simd")]
use std::simd::{StdFloat, cmp::SimdPartialOrd, f32x4, f64x4, num::SimdFloat};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Number of SIMD lanes used for vectorized operations.
#[cfg(feature = "simd")]
pub const SIMD_LANES: usize = 4;

/// A floating-point type suitable for audio sample data.
///
/// This trait abstracts over `f32` and `f64`, allowing DSP blocks and graphs
/// to be generic over sample precision. Use `f32` for performance-critical
/// real-time processing, or `f64` when higher precision is required.
///
/// When the `simd` feature is enabled, this trait also provides associated
/// types and methods for SIMD vectorization.
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
    /// The zero value for this sample type (silence).
    const ZERO: Self;

    /// The unit value for this sample type (full scale).
    const ONE: Self;

    /// Machine epsilon, which is the difference between
    /// 1.0 and the next larger representable number.
    const EPSILON: Self;

    /// Pi (π).
    const PI: Self;

    /// The reciprocal of pi (1/π).
    const INV_PI: Self;

    /// Half of pi (π/2).
    const FRAC_PI_2: Self;

    /// Third of pi (π/3).
    const FRAC_PI_3: Self;

    /// Quarter of pi (π/4).
    const FRAC_PI_4: Self;

    /// Tau; full circle constant (τ = 2π).
    const TAU: Self;

    /// Inverse tau (1/τ = 1/2π)
    const INV_TAU: Self;

    /// The golden ratio (φ).
    const PHI: Self;

    /// Euler's number (e).
    const E: Self;

    /// Square root of 2.
    const SQRT_2: Self;

    /// Inverse square root of 2.
    const INV_SQRT_2: Self;

    /// Convert from an `f64` value.
    fn from_f64(value: f64) -> Self;

    /// Convert to an `f64` value.
    fn to_f64(self) -> f64;

    /// The SIMD vector type for this sample type.
    ///
    /// This associated type provides all necessary SIMD operations for DSP processing.
    #[cfg(feature = "simd")]
    type Simd: SimdFloat<Scalar = Self>
        + StdFloat
        + SimdPartialOrd
        + Copy
        + Add<Output = Self::Simd>
        + Sub<Output = Self::Simd>
        + Mul<Output = Self::Simd>
        + Div<Output = Self::Simd>
        + std::ops::Rem<Output = Self::Simd>;

    /// Create a SIMD vector with all lanes set to the given value.
    #[cfg(feature = "simd")]
    fn simd_splat(value: Self) -> Self::Simd;

    /// Load a SIMD vector from a slice (must have at least SIMD_LANES elements).
    #[cfg(feature = "simd")]
    fn simd_from_slice(slice: &[Self]) -> Self::Simd;

    /// Convert a SIMD vector to an array.
    #[cfg(feature = "simd")]
    fn simd_to_array(simd: Self::Simd) -> [Self; SIMD_LANES];

    /// Select elements based on a greater-than comparison.
    /// Returns `if_true[i]` where `a[i] > b[i]`, otherwise `if_false[i]`.
    #[cfg(feature = "simd")]
    fn simd_select_gt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd;

    /// Select elements based on a less-than comparison.
    /// Returns `if_true[i]` where `a[i] < b[i]`, otherwise `if_false[i]`.
    #[cfg(feature = "simd")]
    fn simd_select_lt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd;

    /// Returns a SIMD vector with lane offsets [0.0, 1.0, 2.0, 3.0].
    #[cfg(feature = "simd")]
    fn simd_lane_offsets() -> Self::Simd;
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = 1.19209290e-07_f32;
    const PI: Self = 3.14159265358979323846264338327950288_f32;
    const INV_PI: Self = 0.318309886183790671537767526745028724_f32;
    const FRAC_PI_2: Self = 1.57079632679489661923132169163975144_f32;
    const FRAC_PI_3: Self = 1.04719755119659774615421446109316763_f32;
    const FRAC_PI_4: Self = 0.785398163397448309615660845819875721_f32;
    const TAU: Self = 6.28318530717958647692528676655900577_f32;
    const INV_TAU: Self = 0.15915494309189533576882414343516084_f32;
    const PHI: Self = 1.618033988749894848204586834365638118_f32;
    const E: Self = 2.71828182845904523536028747135266250_f32;
    const SQRT_2: Self = 1.41421356237309504880168872420969808_f32;
    const INV_SQRT_2: Self = 0.707106781186547524400844362104849039_f32;

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as f32
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[cfg(feature = "simd")]
    type Simd = f32x4;

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_splat(value: Self) -> Self::Simd {
        f32x4::splat(value)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_from_slice(slice: &[Self]) -> Self::Simd {
        f32x4::from_slice(slice)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_to_array(simd: Self::Simd) -> [Self; SIMD_LANES] {
        simd.to_array()
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_select_gt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd {
        a.simd_gt(b).select(if_true, if_false)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_select_lt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd {
        a.simd_lt(b).select(if_true, if_false)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_lane_offsets() -> Self::Simd {
        f32x4::from_array([0.0, 1.0, 2.0, 3.0])
    }
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = 2.2204460492503131e-16_f64;
    const PI: Self = 3.14159265358979323846264338327950288_f64;
    const INV_PI: Self = 0.318309886183790671537767526745028724_f64;
    const FRAC_PI_2: Self = 1.57079632679489661923132169163975144_f64;
    const FRAC_PI_3: Self = 1.04719755119659774615421446109316763_f64;
    const FRAC_PI_4: Self = 0.785398163397448309615660845819875721_f64;
    const TAU: Self = 6.28318530717958647692528676655900577_f64;
    const INV_TAU: Self = 0.15915494309189533576882414343516084_f64;
    const PHI: Self = 1.618033988749894848204586834365638118_f64;
    const E: Self = 2.71828182845904523536028747135266250_f64;
    const SQRT_2: Self = 1.41421356237309504880168872420969808_f64;
    const INV_SQRT_2: Self = 0.707106781186547524400844362104849039_f64;

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }

    #[cfg(feature = "simd")]
    type Simd = f64x4;

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_splat(value: Self) -> Self::Simd {
        f64x4::splat(value)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_from_slice(slice: &[Self]) -> Self::Simd {
        f64x4::from_slice(slice)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_to_array(simd: Self::Simd) -> [Self; SIMD_LANES] {
        simd.to_array()
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_select_gt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd {
        a.simd_gt(b).select(if_true, if_false)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_select_lt(a: Self::Simd, b: Self::Simd, if_true: Self::Simd, if_false: Self::Simd) -> Self::Simd {
        a.simd_lt(b).select(if_true, if_false)
    }

    #[cfg(feature = "simd")]
    #[inline]
    fn simd_lane_offsets() -> Self::Simd {
        f64x4::from_array([0.0, 1.0, 2.0, 3.0])
    }
}
