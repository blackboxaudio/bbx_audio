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

    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }
    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
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
