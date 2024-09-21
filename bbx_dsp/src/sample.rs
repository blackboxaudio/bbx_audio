use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::float::Float;

pub trait Sample:
    Copy
    + Clone
    + From<Self::Float>
    + PartialEq
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

    #[inline]
    fn sqrt(self) -> Self {
        todo!()
    }

    fn powf(self, exp: Self) -> Self;

    fn min(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
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
    fn powf(self, exp: Self) -> Self {
        Float::powf(self, exp)
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        Float::min(self, other)
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        Float::max(self, other)
    }
}
