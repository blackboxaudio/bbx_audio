use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Describes the underlying data type used in audio-related
/// calculations within the DSP graph.
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
    const ZERO: Self;
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
