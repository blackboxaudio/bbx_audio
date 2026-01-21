//! Audio sample type abstraction.
//!
//! This module defines the [`Sample`] trait, which abstracts over floating-point
//! types used for audio processing. This allows blocks and graphs to be generic
//! over sample precision (`f32` or `f64`).

#![allow(clippy::excessive_precision)]

#[cfg(feature = "simd")]
use core::simd::{cmp::SimdPartialOrd, f32x4, f64x4, num::SimdFloat};
use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
#[cfg(feature = "simd")]
use std::simd::StdFloat;

use crate::math::Real;

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
    + Real
    + 'static
{
    /// The zero value for this sample type (silence).
    const ZERO: Self;

    /// The unit value for this sample type (full scale).
    const ONE: Self;

    /// Machine epsilon i.e. the difference between
    /// 1.0 and the next larger representable number.
    const EPSILON: Self;

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
        + core::ops::Rem<Output = Self::Simd>;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq<S: Sample>(a: S, b: S, epsilon: f64) -> bool {
        (a.to_f64() - b.to_f64()).abs() < epsilon
    }

    #[test]
    fn test_zero_and_one() {
        assert_eq!(f32::ZERO, 0.0f32);
        assert_eq!(f32::ONE, 1.0f32);
        assert_eq!(f64::ZERO, 0.0f64);
        assert_eq!(f64::ONE, 1.0f64);
    }

    #[test]
    fn test_epsilon_is_small_positive() {
        assert!(f32::EPSILON > 0.0);
        assert!(f32::EPSILON < 1e-5);
        assert!(f64::EPSILON > 0.0);
        assert!(f64::EPSILON < 1e-14);
    }

    #[test]
    fn test_f32_to_f64_conversion() {
        let values = [0.0f32, 1.0, -1.0, 0.5, -0.5, f32::EPSILON];
        for v in values {
            let converted = v.to_f64();
            assert!((converted - v as f64).abs() < 1e-7);
        }
    }

    #[test]
    fn test_f64_to_f64_identity() {
        let values = [0.0f64, 1.0, -1.0, 0.5, f64::EPSILON, std::f64::consts::PI];
        for v in values {
            assert_eq!(v.to_f64(), v);
        }
    }

    #[test]
    fn test_from_f64_f32() {
        let values = [0.0f64, 1.0, -1.0, 0.5, -0.5];
        for v in values {
            let converted = f32::from_f64(v);
            assert!((converted as f64 - v).abs() < 1e-6);
        }
    }

    #[test]
    fn test_from_f64_f64_identity() {
        let values = [0.0f64, 1.0, -1.0, std::f64::consts::PI];
        for v in values {
            assert_eq!(f64::from_f64(v), v);
        }
    }

    #[test]
    fn test_roundtrip_f32() {
        let values = [0.0f32, 1.0, -1.0, 0.5, 0.123456];
        for v in values {
            let roundtrip = f32::from_f64(v.to_f64());
            assert!((roundtrip - v).abs() < f32::EPSILON * 2.0);
        }
    }

    #[test]
    fn test_infinity_handling_f32() {
        let pos_inf = f32::INFINITY;
        let neg_inf = f32::NEG_INFINITY;
        assert!(pos_inf.to_f64().is_infinite());
        assert!(neg_inf.to_f64().is_infinite());
        assert!(pos_inf.to_f64() > 0.0);
        assert!(neg_inf.to_f64() < 0.0);
    }

    #[test]
    fn test_infinity_handling_f64() {
        let pos_inf = f64::INFINITY;
        let neg_inf = f64::NEG_INFINITY;
        assert!(pos_inf.to_f64().is_infinite());
        assert!(neg_inf.to_f64().is_infinite());
        assert_eq!(f64::from_f64(pos_inf), f64::INFINITY);
        assert_eq!(f64::from_f64(neg_inf), f64::NEG_INFINITY);
    }

    #[test]
    fn test_nan_propagation_f32() {
        let nan = f32::NAN;
        assert!(nan.to_f64().is_nan());
        assert!(f32::from_f64(f64::NAN).is_nan());
    }

    #[test]
    fn test_nan_propagation_f64() {
        let nan = f64::NAN;
        assert!(nan.to_f64().is_nan());
        assert!(f64::from_f64(f64::NAN).is_nan());
    }

    #[test]
    fn test_denormal_conversion_f32() {
        let denormal = 1e-40_f32;
        assert!(denormal != 0.0);
        let converted = denormal.to_f64();
        assert!(converted != 0.0);
        assert!((converted - denormal as f64).abs() < 1e-45);
    }

    #[test]
    fn test_denormal_conversion_f64() {
        let denormal = 1e-310_f64;
        assert!(denormal != 0.0);
        assert_eq!(denormal.to_f64(), denormal);
    }

    #[test]
    fn test_arithmetic_operations_f32() {
        let a: f32 = 2.0;
        let b: f32 = 3.0;
        assert!(approx_eq(a + b, 5.0f32, 1e-6));
        assert!(approx_eq(a - b, -1.0f32, 1e-6));
        assert!(approx_eq(a * b, 6.0f32, 1e-6));
        assert!(approx_eq(a / b, 2.0 / 3.0, 1e-6));
        assert!(approx_eq(-a, -2.0f32, 1e-6));
    }

    #[test]
    fn test_arithmetic_operations_f64() {
        let a: f64 = 2.0;
        let b: f64 = 3.0;
        assert!(approx_eq(a + b, 5.0f64, 1e-14));
        assert!(approx_eq(a - b, -1.0f64, 1e-14));
        assert!(approx_eq(a * b, 6.0f64, 1e-14));
        assert!(approx_eq(a / b, 2.0 / 3.0, 1e-14));
        assert!(approx_eq(-a, -2.0f64, 1e-14));
    }

    #[test]
    fn test_assign_operations_f32() {
        let mut v: f32 = 1.0;
        v += 2.0;
        assert!(approx_eq(v, 3.0f32, 1e-6));
        v -= 1.0;
        assert!(approx_eq(v, 2.0f32, 1e-6));
        v *= 3.0;
        assert!(approx_eq(v, 6.0f32, 1e-6));
        v /= 2.0;
        assert!(approx_eq(v, 3.0f32, 1e-6));
    }

    #[test]
    fn test_assign_operations_f64() {
        let mut v: f64 = 1.0;
        v += 2.0;
        assert!(approx_eq(v, 3.0f64, 1e-14));
        v -= 1.0;
        assert!(approx_eq(v, 2.0f64, 1e-14));
        v *= 3.0;
        assert!(approx_eq(v, 6.0f64, 1e-14));
        v /= 2.0;
        assert!(approx_eq(v, 3.0f64, 1e-14));
    }

    #[test]
    fn test_comparison_operations() {
        assert!(1.0f32 < 2.0f32);
        assert!(2.0f32 > 1.0f32);
        assert!(1.0f32 <= 1.0f32);
        assert!(1.0f32 >= 1.0f32);
        assert!(1.0f32 == 1.0f32);
        assert!(1.0f32 != 2.0f32);

        assert!(1.0f64 < 2.0f64);
        assert!(2.0f64 > 1.0f64);
    }

    #[cfg(feature = "simd")]
    mod simd_tests {
        use super::*;

        #[test]
        fn test_simd_splat_f32() {
            let vec = f32::simd_splat(2.5);
            let arr = f32::simd_to_array(vec);
            for v in arr {
                assert!((v - 2.5).abs() < 1e-6);
            }
        }

        #[test]
        fn test_simd_splat_f64() {
            let vec = f64::simd_splat(2.5);
            let arr = f64::simd_to_array(vec);
            for v in arr {
                assert!((v - 2.5).abs() < 1e-14);
            }
        }

        #[test]
        fn test_simd_from_slice_f32() {
            let data = [1.0f32, 2.0, 3.0, 4.0];
            let vec = f32::simd_from_slice(&data);
            let arr = f32::simd_to_array(vec);
            for (a, b) in arr.iter().zip(data.iter()) {
                assert!((a - b).abs() < 1e-6);
            }
        }

        #[test]
        fn test_simd_from_slice_f64() {
            let data = [1.0f64, 2.0, 3.0, 4.0];
            let vec = f64::simd_from_slice(&data);
            let arr = f64::simd_to_array(vec);
            for (a, b) in arr.iter().zip(data.iter()) {
                assert!((a - b).abs() < 1e-14);
            }
        }

        #[test]
        fn test_simd_lane_offsets_f32() {
            let offsets = f32::simd_lane_offsets();
            let arr = f32::simd_to_array(offsets);
            assert!((arr[0] - 0.0).abs() < 1e-6);
            assert!((arr[1] - 1.0).abs() < 1e-6);
            assert!((arr[2] - 2.0).abs() < 1e-6);
            assert!((arr[3] - 3.0).abs() < 1e-6);
        }

        #[test]
        fn test_simd_lane_offsets_f64() {
            let offsets = f64::simd_lane_offsets();
            let arr = f64::simd_to_array(offsets);
            assert!((arr[0] - 0.0).abs() < 1e-14);
            assert!((arr[1] - 1.0).abs() < 1e-14);
            assert!((arr[2] - 2.0).abs() < 1e-14);
            assert!((arr[3] - 3.0).abs() < 1e-14);
        }

        #[test]
        fn test_simd_select_gt_f32() {
            let a = f32::simd_from_slice(&[1.0, 3.0, 2.0, 5.0]);
            let b = f32::simd_from_slice(&[2.0, 2.0, 2.0, 2.0]);
            let if_true = f32::simd_splat(10.0);
            let if_false = f32::simd_splat(0.0);
            let result = f32::simd_select_gt(a, b, if_true, if_false);
            let arr = f32::simd_to_array(result);
            assert!((arr[0] - 0.0).abs() < 1e-6);
            assert!((arr[1] - 10.0).abs() < 1e-6);
            assert!((arr[2] - 0.0).abs() < 1e-6);
            assert!((arr[3] - 10.0).abs() < 1e-6);
        }

        #[test]
        fn test_simd_select_lt_f32() {
            let a = f32::simd_from_slice(&[1.0, 3.0, 2.0, 5.0]);
            let b = f32::simd_from_slice(&[2.0, 2.0, 2.0, 2.0]);
            let if_true = f32::simd_splat(10.0);
            let if_false = f32::simd_splat(0.0);
            let result = f32::simd_select_lt(a, b, if_true, if_false);
            let arr = f32::simd_to_array(result);
            assert!((arr[0] - 10.0).abs() < 1e-6);
            assert!((arr[1] - 0.0).abs() < 1e-6);
            assert!((arr[2] - 0.0).abs() < 1e-6);
            assert!((arr[3] - 0.0).abs() < 1e-6);
        }

        #[test]
        fn test_simd_select_gt_f64() {
            let a = f64::simd_from_slice(&[1.0, 3.0, 2.0, 5.0]);
            let b = f64::simd_from_slice(&[2.0, 2.0, 2.0, 2.0]);
            let if_true = f64::simd_splat(10.0);
            let if_false = f64::simd_splat(0.0);
            let result = f64::simd_select_gt(a, b, if_true, if_false);
            let arr = f64::simd_to_array(result);
            assert!((arr[0] - 0.0).abs() < 1e-14);
            assert!((arr[1] - 10.0).abs() < 1e-14);
            assert!((arr[2] - 0.0).abs() < 1e-14);
            assert!((arr[3] - 10.0).abs() < 1e-14);
        }

        #[test]
        fn test_simd_select_lt_f64() {
            let a = f64::simd_from_slice(&[1.0, 3.0, 2.0, 5.0]);
            let b = f64::simd_from_slice(&[2.0, 2.0, 2.0, 2.0]);
            let if_true = f64::simd_splat(10.0);
            let if_false = f64::simd_splat(0.0);
            let result = f64::simd_select_lt(a, b, if_true, if_false);
            let arr = f64::simd_to_array(result);
            assert!((arr[0] - 10.0).abs() < 1e-14);
            assert!((arr[1] - 0.0).abs() < 1e-14);
            assert!((arr[2] - 0.0).abs() < 1e-14);
            assert!((arr[3] - 0.0).abs() < 1e-14);
        }
    }
}
