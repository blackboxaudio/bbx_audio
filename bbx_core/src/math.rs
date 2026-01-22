//! Mathematical operations using libm for consistent results across std and no_std builds.
//!
//! This module provides a [`Real`] trait that abstracts over floating-point types
//! and uses `libm` for all mathematical operations, ensuring identical audio processing
//! results regardless of the build configuration.

#![allow(clippy::approx_constant)]
#![allow(clippy::excessive_precision)]

/// Trait for real number types supporting mathematical operations via libm.
///
/// All implementations use `libm` functions directly, ensuring consistent behavior
/// between std and no_std builds.
pub trait Real: Copy {
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

    /// Inverse tau (1/τ = 1/2π).
    const INV_TAU: Self;

    /// The golden ratio (φ).
    const PHI: Self;

    /// Euler's number (e).
    const E: Self;

    /// Square root of 2.
    const SQRT_2: Self;

    /// Inverse square root of 2.
    const INV_SQRT_2: Self;

    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn ln(self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn powf(self, exp: Self) -> Self;
    fn sqrt(self) -> Self;
    fn cbrt(self) -> Self;
    fn abs(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn copysign(self, sign: Self) -> Self;
    fn radians(self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
}

impl Real for f32 {
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
    fn sin(self) -> Self {
        libm::sinf(self)
    }
    #[inline]
    fn cos(self) -> Self {
        libm::cosf(self)
    }
    #[inline]
    fn tan(self) -> Self {
        libm::tanf(self)
    }
    #[inline]
    fn asin(self) -> Self {
        libm::asinf(self)
    }
    #[inline]
    fn acos(self) -> Self {
        libm::acosf(self)
    }
    #[inline]
    fn atan(self) -> Self {
        libm::atanf(self)
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2f(self, other)
    }
    #[inline]
    fn sinh(self) -> Self {
        libm::sinhf(self)
    }
    #[inline]
    fn cosh(self) -> Self {
        libm::coshf(self)
    }
    #[inline]
    fn tanh(self) -> Self {
        libm::tanhf(self)
    }
    #[inline]
    fn exp(self) -> Self {
        libm::expf(self)
    }
    #[inline]
    fn exp2(self) -> Self {
        libm::exp2f(self)
    }
    #[inline]
    fn ln(self) -> Self {
        libm::logf(self)
    }
    #[inline]
    fn log2(self) -> Self {
        libm::log2f(self)
    }
    #[inline]
    fn log10(self) -> Self {
        libm::log10f(self)
    }
    #[inline]
    fn powf(self, exp: Self) -> Self {
        libm::powf(self, exp)
    }
    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }
    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrtf(self)
    }
    #[inline]
    fn abs(self) -> Self {
        libm::fabsf(self)
    }
    #[inline]
    fn floor(self) -> Self {
        libm::floorf(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }
    #[inline]
    fn round(self) -> Self {
        libm::roundf(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::truncf(self)
    }
    #[inline]
    fn fract(self) -> Self {
        self - libm::floorf(self)
    }
    #[inline]
    fn copysign(self, sign: Self) -> Self {
        libm::copysignf(self, sign)
    }
    #[inline]
    fn radians(self) -> Self {
        self * Self::PI / 180.0
    }
    #[inline]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = libm::fmodf(self, rhs);
        if r < 0.0 { r + libm::fabsf(rhs) } else { r }
    }
}

impl Real for f64 {
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
    fn sin(self) -> Self {
        libm::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        libm::cos(self)
    }
    #[inline]
    fn tan(self) -> Self {
        libm::tan(self)
    }
    #[inline]
    fn asin(self) -> Self {
        libm::asin(self)
    }
    #[inline]
    fn acos(self) -> Self {
        libm::acos(self)
    }
    #[inline]
    fn atan(self) -> Self {
        libm::atan(self)
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2(self, other)
    }
    #[inline]
    fn sinh(self) -> Self {
        libm::sinh(self)
    }
    #[inline]
    fn cosh(self) -> Self {
        libm::cosh(self)
    }
    #[inline]
    fn tanh(self) -> Self {
        libm::tanh(self)
    }
    #[inline]
    fn exp(self) -> Self {
        libm::exp(self)
    }
    #[inline]
    fn exp2(self) -> Self {
        libm::exp2(self)
    }
    #[inline]
    fn ln(self) -> Self {
        libm::log(self)
    }
    #[inline]
    fn log2(self) -> Self {
        libm::log2(self)
    }
    #[inline]
    fn log10(self) -> Self {
        libm::log10(self)
    }
    #[inline]
    fn powf(self, exp: Self) -> Self {
        libm::pow(self, exp)
    }
    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }
    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrt(self)
    }
    #[inline]
    fn abs(self) -> Self {
        libm::fabs(self)
    }
    #[inline]
    fn floor(self) -> Self {
        libm::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }
    #[inline]
    fn round(self) -> Self {
        libm::round(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::trunc(self)
    }
    #[inline]
    fn fract(self) -> Self {
        self - libm::floor(self)
    }
    #[inline]
    fn copysign(self, sign: Self) -> Self {
        libm::copysign(self, sign)
    }
    #[inline]
    fn radians(self) -> Self {
        self * Self::PI / 180.0
    }
    #[inline]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = libm::fmod(self, rhs);
        if r < 0.0 { r + libm::fabs(rhs) } else { r }
    }
}

// Generic functions for ergonomic usage

#[inline]
pub fn sin<T: Real>(x: T) -> T {
    x.sin()
}

#[inline]
pub fn cos<T: Real>(x: T) -> T {
    x.cos()
}

#[inline]
pub fn tan<T: Real>(x: T) -> T {
    x.tan()
}

#[inline]
pub fn asin<T: Real>(x: T) -> T {
    x.asin()
}

#[inline]
pub fn acos<T: Real>(x: T) -> T {
    x.acos()
}

#[inline]
pub fn atan<T: Real>(x: T) -> T {
    x.atan()
}

#[inline]
pub fn atan2<T: Real>(y: T, x: T) -> T {
    y.atan2(x)
}

#[inline]
pub fn sinh<T: Real>(x: T) -> T {
    x.sinh()
}

#[inline]
pub fn cosh<T: Real>(x: T) -> T {
    x.cosh()
}

#[inline]
pub fn tanh<T: Real>(x: T) -> T {
    x.tanh()
}

#[inline]
pub fn exp<T: Real>(x: T) -> T {
    x.exp()
}

#[inline]
pub fn exp2<T: Real>(x: T) -> T {
    x.exp2()
}

#[inline]
pub fn ln<T: Real>(x: T) -> T {
    x.ln()
}

#[inline]
pub fn log2<T: Real>(x: T) -> T {
    x.log2()
}

#[inline]
pub fn log10<T: Real>(x: T) -> T {
    x.log10()
}

#[inline]
pub fn powf<T: Real>(base: T, exp: T) -> T {
    base.powf(exp)
}

#[inline]
pub fn sqrt<T: Real>(x: T) -> T {
    x.sqrt()
}

#[inline]
pub fn cbrt<T: Real>(x: T) -> T {
    x.cbrt()
}

#[inline]
pub fn abs<T: Real>(x: T) -> T {
    x.abs()
}

#[inline]
pub fn floor<T: Real>(x: T) -> T {
    x.floor()
}

#[inline]
pub fn ceil<T: Real>(x: T) -> T {
    x.ceil()
}

#[inline]
pub fn round<T: Real>(x: T) -> T {
    x.round()
}

#[inline]
pub fn trunc<T: Real>(x: T) -> T {
    x.trunc()
}

#[inline]
pub fn fract<T: Real>(x: T) -> T {
    x.fract()
}

#[inline]
pub fn copysign<T: Real>(magnitude: T, sign: T) -> T {
    magnitude.copysign(sign)
}

#[inline]
pub fn radians<T: Real>(deg: T) -> T {
    deg.radians()
}

#[inline]
pub fn rem_euclid<T: Real>(x: T, rhs: T) -> T {
    x.rem_euclid(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_f32_constants_accuracy() {
        let epsilon = 1e-6;
        assert!(approx_eq(f32::PI as f64, std::f32::consts::PI as f64, epsilon));
        assert!(approx_eq(f32::TAU as f64, std::f32::consts::TAU as f64, epsilon));
        assert!(approx_eq(f32::E as f64, std::f32::consts::E as f64, epsilon));
        assert!(approx_eq(f32::SQRT_2 as f64, std::f32::consts::SQRT_2 as f64, epsilon));
        assert!(approx_eq(
            f32::FRAC_PI_2 as f64,
            std::f32::consts::FRAC_PI_2 as f64,
            epsilon
        ));
        assert!(approx_eq(
            f32::FRAC_PI_3 as f64,
            std::f32::consts::FRAC_PI_3 as f64,
            epsilon
        ));
        assert!(approx_eq(
            f32::FRAC_PI_4 as f64,
            std::f32::consts::FRAC_PI_4 as f64,
            epsilon
        ));
    }

    #[test]
    fn test_f64_constants_accuracy() {
        let epsilon = 1e-14;
        assert!(approx_eq(f64::PI, std::f64::consts::PI, epsilon));
        assert!(approx_eq(f64::TAU, std::f64::consts::TAU, epsilon));
        assert!(approx_eq(f64::E, std::f64::consts::E, epsilon));
        assert!(approx_eq(f64::SQRT_2, std::f64::consts::SQRT_2, epsilon));
        assert!(approx_eq(f64::FRAC_PI_2, std::f64::consts::FRAC_PI_2, epsilon));
        assert!(approx_eq(f64::FRAC_PI_3, std::f64::consts::FRAC_PI_3, epsilon));
        assert!(approx_eq(f64::FRAC_PI_4, std::f64::consts::FRAC_PI_4, epsilon));
    }

    #[test]
    fn test_derived_constants_f32() {
        let epsilon = 1e-6;
        assert!(approx_eq(
            f32::INV_PI as f64,
            (1.0 / std::f32::consts::PI) as f64,
            epsilon
        ));
        assert!(approx_eq(
            f32::INV_TAU as f64,
            (1.0 / std::f32::consts::TAU) as f64,
            epsilon
        ));
        assert!(approx_eq(
            f32::INV_SQRT_2 as f64,
            (1.0 / std::f32::consts::SQRT_2) as f64,
            epsilon
        ));
    }

    #[test]
    fn test_derived_constants_f64() {
        let epsilon = 1e-14;
        assert!(approx_eq(f64::INV_PI, 1.0 / std::f64::consts::PI, epsilon));
        assert!(approx_eq(f64::INV_TAU, 1.0 / std::f64::consts::TAU, epsilon));
        assert!(approx_eq(f64::INV_SQRT_2, 1.0 / std::f64::consts::SQRT_2, epsilon));
    }

    #[test]
    fn test_phi_golden_ratio() {
        let epsilon_f32 = 1e-6;
        let epsilon_f64 = 1e-14;
        let expected_phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        assert!(approx_eq(f32::PHI as f64, expected_phi, epsilon_f32));
        assert!(approx_eq(f64::PHI, expected_phi, epsilon_f64));
    }
}
