//! Audio sample type abstraction.
//!
//! This module defines the [`Sample`] trait, which abstracts over floating-point
//! types used for audio processing. This allows blocks and graphs to be generic
//! over sample precision (`f32` or `f64`).

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
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

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
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

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
}
