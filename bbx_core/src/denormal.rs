//! Denormal (subnormal) float handling utilities.
//!
//! Denormalized floating-point numbers are very small values that can cause
//! significant CPU slowdowns (10-100x) on x86 processors due to microcode
//! fallback handling. This module provides utilities to flush these values
//! to zero, preventing performance degradation in audio processing.
//!
//! When the `ftz-daz` feature is enabled, this module also provides CPU-level
//! FTZ (Flush-To-Zero) and DAZ (Denormals-Are-Zero) mode configuration for
//! x86/x86_64 processors.

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

/// Enable FTZ (Flush-To-Zero) and DAZ (Denormals-Are-Zero) modes on x86/x86_64.
///
/// This sets CPU flags that cause denormalized floating-point numbers to be
/// automatically flushed to zero, avoiding the significant performance penalty
/// (10-100x slowdown) that denormals can cause.
///
/// This function is only available when the `ftz-daz` feature is enabled and
/// compiling for x86/x86_64 targets.
///
/// # Safety
///
/// This function modifies CPU control registers. It is safe to call from any
/// thread, but the flags are per-thread on most systems. Call this at the start
/// of any audio processing thread.
#[cfg(all(feature = "ftz-daz", any(target_arch = "x86", target_arch = "x86_64")))]
pub fn enable_ftz_daz() {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::{_MM_FLUSH_ZERO_ON, _MM_SET_FLUSH_ZERO_MODE, _mm_getcsr, _mm_setcsr};
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::{_MM_FLUSH_ZERO_ON, _MM_SET_FLUSH_ZERO_MODE, _mm_getcsr, _mm_setcsr};

    const DAZ_BIT: u32 = 1 << 6;

    unsafe {
        // Enable FTZ (Flush-To-Zero)
        _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);

        // Enable DAZ (Denormals-Are-Zero)
        let mxcsr = _mm_getcsr();
        _mm_setcsr(mxcsr | DAZ_BIT);
    }
}

/// No-op stub for non-x86 architectures when `ftz-daz` feature is enabled.
#[cfg(all(feature = "ftz-daz", not(any(target_arch = "x86", target_arch = "x86_64"))))]
pub fn enable_ftz_daz() {
    // No-op on non-x86 architectures
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
