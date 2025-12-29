//! Audio sample type abstraction.
//!
//! This module defines the [`Sample`] trait, which abstracts over floating-point
//! types used for audio processing. This allows blocks and graphs to be generic
//! over sample precision (`f32` or `f64`).

use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// A floating-point type suitable for audio sample data.
///
/// This trait abstracts over `f32` and `f64`, allowing DSP blocks and graphs
/// to be generic over sample precision. Use `f32` for performance-critical
/// real-time processing, or `f64` when higher precision is required.
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
}
