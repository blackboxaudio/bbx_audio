use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::float::Float;

/// The unit of data within an audio DSP system. In this case it wraps
/// any data type that is trait-bound to `Float`.
pub trait Sample:
    Copy
    + Clone
    + From<Self::Float>
    + PartialEq
    + PartialOrd
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    type Float: Float;

    const EQUILIBRIUM: Self;

    fn from_f32(f: f32) -> Self;

    fn apply<F: Fn(Self::Float) -> Self::Float>(self, f: F) -> Self
    where
        Self: Sized;

    #[inline]
    fn gain(&self, factor: Self::Float) -> Self {
        let linear = Self::Float::powf(Self::Float::from(10.0), factor / Self::Float::from(10.0));

        self.apply(|f| f * linear)
    }

    #[inline]
    fn sin(self) -> Self {
        self.apply(Float::sin)
    }

    #[inline]
    fn cos(self) -> Self {
        self.apply(Float::cos)
    }

    #[inline]
    fn tan(self) -> Self {
        self.apply(Float::tan)
    }

    #[inline]
    fn atan(self) -> Self {
        self.apply(Float::atan)
    }

    fn sqrt(self) -> Self;

    fn powf(self, exp: Self) -> Self;

    #[inline]
    fn min(self, rhs: Self) -> Self {
        if self < rhs {
            self
        } else {
            rhs
        }
    }

    #[inline]
    fn max(self, rhs: Self) -> Self {
        if self < rhs {
            rhs
        } else {
            self
        }
    }
}

impl Sample for f32 {
    type Float = f32;

    const EQUILIBRIUM: Self = 0.0;

    #[inline]
    fn from_f32(f: f32) -> Self {
        f
    }

    fn apply<F: Fn(Self::Float) -> Self::Float>(self, f: F) -> Self
    where
        Self: Sized,
    {
        f(self)
    }

    #[inline]
    fn sqrt(self) -> Self {
        let mut i: i32 = self.to_bits() as i32;
        i = 0x5f3759df - (i >> 1);
        let y = f32::from_bits(i as u32);
        let inv = y * (1.5 - (self * 0.5 * y * y));
        1.0 / inv
    }

    #[inline]
    fn powf(self, exp: Self) -> Self {
        Float::powf(self, exp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_f32() {
        assert_eq!(f32::from_f32(3.5), 3.5);
    }

    #[test]
    fn test_apply() {
        let value: f32 = 2.0;
        let result = value.apply(|x| x * 2.0);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_gain() {
        let value: f32 = 1.0;
        let factor: f32 = 2.0;
        let result = value.gain(factor);
        assert_eq!(result, 1.5848932);
    }

    #[test]
    fn test_sin() {
        let value: f32 = std::f32::consts::PI / 2.0;
        assert_eq!(value.sin(), 1.0);
    }

    #[test]
    fn test_cos() {
        let value: f32 = std::f32::consts::PI;
        assert_eq!(value.cos(), -1.0);
    }

    #[test]
    fn test_tan() {
        let value: f32 = std::f32::consts::PI / 4.0;
        assert!((value.tan() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_atan() {
        let value: f32 = 1.0;
        assert!((value.atan() - (std::f32::consts::PI / 4.0)).abs() < 1e-6);
    }

    #[test]
    fn test_sqrt() {
        let value: f32 = 4.0;
        let result = value.sqrt();
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_powf() {
        let base: f32 = 2.0;
        let exp: f32 = 3.0;
        assert_eq!(base.powf(exp), 8.0);
    }

    #[test]
    fn test_min() {
        let a: f32 = 3.0;
        let b: f32 = 5.0;
        assert_eq!(a.min(b), a);
    }

    #[test]
    fn test_max() {
        let a: f32 = 3.0;
        let b: f32 = 5.0;
        assert_eq!(a.max(b), b);
    }
}
