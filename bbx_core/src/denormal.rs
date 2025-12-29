//! Denormal (subnormal) float handling utilities.
//!
//! Denormalized floating-point numbers are very small values that can cause
//! significant CPU slowdowns (10-100x) on x86 processors due to microcode
//! fallback handling. This module provides utilities to flush these values
//! to zero, preventing performance degradation in audio processing.

/// Threshold below which values are considered denormal.
/// This is slightly above the actual denormal threshold to catch
/// values that will become denormal after further processing.
pub const DENORMAL_THRESHOLD_F64: f64 = 1e-15;
pub const DENORMAL_THRESHOLD_F32: f32 = 1e-15;

/// Flush a denormal f64 value to zero.
///
/// Values with absolute value below the threshold are replaced with zero
/// to prevent CPU slowdowns from denormalized float handling.
#[inline]
pub fn flush_denormal_f64(x: f64) -> f64 {
    if x.abs() < DENORMAL_THRESHOLD_F64 { 0.0 } else { x }
}

/// Flush a denormal f32 value to zero.
#[inline]
pub fn flush_denormal_f32(x: f32) -> f32 {
    if x.abs() < DENORMAL_THRESHOLD_F32 { 0.0 } else { x }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_values_unchanged() {
        assert_eq!(flush_denormal_f64(1.0), 1.0);
        assert_eq!(flush_denormal_f64(-0.5), -0.5);
        assert_eq!(flush_denormal_f64(1e-10), 1e-10);
    }

    #[test]
    fn test_denormal_flushed_to_zero() {
        assert_eq!(flush_denormal_f64(1e-16), 0.0);
        assert_eq!(flush_denormal_f64(-1e-16), 0.0);
        assert_eq!(flush_denormal_f64(1e-300), 0.0);
    }

    #[test]
    fn test_zero_unchanged() {
        assert_eq!(flush_denormal_f64(0.0), 0.0);
        assert_eq!(flush_denormal_f64(-0.0), 0.0);
    }

    #[test]
    fn test_f32_denormal_handling() {
        assert_eq!(flush_denormal_f32(1.0), 1.0);
        assert_eq!(flush_denormal_f32(1e-16), 0.0);
    }
}
