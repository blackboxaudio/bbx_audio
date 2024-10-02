use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// The basic unit of data most commonly used in DSP operations.
pub trait Float:
    Copy
    + Clone
    + From<f32>
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    const ZERO: Self;
    const IDENTITY: Self;
    const MIN: Self;
    const MAX: Self;
    const PI: Self;
    const EULER: Self;

    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn atan(self) -> Self;

    fn avg(self, other: Self) -> Self;
    fn powf(self, exp: Self) -> Self;
    fn log(self, base: Self) -> Self;
    fn log10(self) -> Self;

    #[inline]
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Float for f32 {
    const ZERO: Self = 0.0;
    const IDENTITY: Self = 1.0;
    const MIN: Self = -1.0;
    const MAX: Self = 1.0;
    const PI: Self = std::f32::consts::PI;
    const EULER: Self = std::f32::consts::E;

    #[inline]
    fn sin(self) -> Self {
        f32::sin(self)
    }

    #[inline]
    fn cos(self) -> Self {
        f32::cos(self)
    }

    #[inline]
    fn tan(self) -> Self {
        f32::tan(self)
    }

    #[inline]
    fn atan(self) -> Self {
        f32::atan(self)
    }

    #[inline]
    fn avg(self, v: Self) -> Self {
        (self + v) / 2.0
    }

    #[inline]
    fn powf(self, e: Self) -> Self {
        f32::powf(self, e)
    }

    #[inline]
    fn log(self, v: Self) -> Self {
        f32::log(self, v)
    }

    #[inline]
    fn log10(self) -> Self {
        f32::log10(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin() {
        let angle = f32::PI / 2.0;
        assert_eq!(angle.sin(), 1.0);
    }

    #[test]
    fn test_cos() {
        let angle = f32::PI;
        assert_eq!(angle.cos(), -1.0);
    }

    #[test]
    fn test_tan() {
        let angle = f32::PI / 4.0;
        assert!((angle.tan() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_atan() {
        let value = 1.0;
        assert!((value.atan() - (f32::PI / 4.0)).abs() < 1e-6);
    }

    #[test]
    fn test_avg() {
        let a = 3.0;
        let b = 5.0;
        assert_eq!(a.avg(b), 4.0);
    }

    #[test]
    fn test_powf() {
        let base = 2.0;
        let exp = 3.0;
        assert_eq!(base.powf(exp), 8.0);
    }

    #[test]
    fn test_log() {
        let value = 8.0;
        let base = 2.0;
        assert_eq!(value.log(base), 3.0);
    }

    #[test]
    fn test_log10() {
        let value = 1000.0;
        assert_eq!(value.log10(), 3.0);
    }

    #[test]
    fn test_min() {
        let a = 3.0;
        let b = 5.0;
        assert_eq!(a.min(b), a);
    }

    #[test]
    fn test_max() {
        let a = 3.0;
        let b = 5.0;
        assert_eq!(a.max(b), b);
    }

    #[test]
    fn test_arithmetic_ops() {
        let a = 3.0;
        let b = 5.0;

        assert_eq!(a + b, 8.0);
        assert_eq!(b - a, 2.0);
        assert_eq!(a * b, 15.0);
        assert_eq!(b / a, 5.0 / 3.0);
    }

    #[test]
    fn test_assign_ops() {
        let mut a = 3.0;
        let b = 5.0;

        a += b;
        assert_eq!(a, 8.0);

        a -= b;
        assert_eq!(a, 3.0);

        a *= b;
        assert_eq!(a, 15.0);

        a /= b;
        assert_eq!(a, 3.0);
    }
}
