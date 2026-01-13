//! Denormal (subnormal) float handling utilities.
//!
//! Denormalized floating-point numbers are very small values that can cause
//! significant CPU slowdowns (10-100x) on x86 processors due to microcode
//! fallback handling. This module provides utilities to flush these values
//! to zero, preventing performance degradation in audio processing.
//!
//! When the `ftz-daz` feature is enabled, this module also provides CPU-level
//! FTZ (Flush-To-Zero) and DAZ (Denormals-Are-Zero) mode configuration for
//! x86/x86_64 and AArch64 processors.

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

/// Batch flush denormals in a slice of f64 values using SIMD.
#[cfg(feature = "simd")]
#[inline]
pub fn flush_denormals_f64_batch(buffer: &mut [f64]) {
    use std::simd::{cmp::SimdPartialOrd, f64x4, num::SimdFloat};

    let threshold = f64x4::splat(DENORMAL_THRESHOLD_F64);
    let zero = f64x4::splat(0.0);

    let (chunks, remainder) = buffer.as_chunks_mut::<4>();
    for chunk in chunks {
        let v = f64x4::from_array(*chunk);
        let mask = v.abs().simd_lt(threshold);
        *chunk = mask.select(zero, v).to_array();
    }
    for sample in remainder {
        *sample = flush_denormal_f64(*sample);
    }
}

/// Batch flush denormals in a slice of f64 values (scalar fallback).
#[cfg(not(feature = "simd"))]
#[inline]
pub fn flush_denormals_f64_batch(buffer: &mut [f64]) {
    for sample in buffer {
        *sample = flush_denormal_f64(*sample);
    }
}

/// Flush a denormal f32 value to zero.
#[inline]
pub fn flush_denormal_f32(x: f32) -> f32 {
    if x.abs() < DENORMAL_THRESHOLD_F32 { 0.0 } else { x }
}

/// Batch flush denormals in a slice of f32 values using SIMD.
#[cfg(feature = "simd")]
#[inline]
pub fn flush_denormals_f32_batch(buffer: &mut [f32]) {
    use std::simd::{cmp::SimdPartialOrd, f32x4, num::SimdFloat};

    let threshold = f32x4::splat(DENORMAL_THRESHOLD_F32);
    let zero = f32x4::splat(0.0);

    let (chunks, remainder) = buffer.as_chunks_mut::<4>();
    for chunk in chunks {
        let v = f32x4::from_array(*chunk);
        let mask = v.abs().simd_lt(threshold);
        *chunk = mask.select(zero, v).to_array();
    }
    for sample in remainder {
        *sample = flush_denormal_f32(*sample);
    }
}

/// Batch flush denormals in a slice of f32 values (scalar fallback).
#[cfg(not(feature = "simd"))]
#[inline]
pub fn flush_denormals_f32_batch(buffer: &mut [f32]) {
    for sample in buffer {
        *sample = flush_denormal_f32(*sample);
    }
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
    use std::arch::asm;

    const FTZ_BIT: u32 = 1 << 15;
    const DAZ_BIT: u32 = 1 << 6;

    unsafe {
        let mut mxcsr: u32 = 0;
        asm!(
            "stmxcsr [{}]",
            in(reg) &mut mxcsr,
            options(nostack, preserves_flags)
        );
        mxcsr |= FTZ_BIT | DAZ_BIT;
        asm!(
            "ldmxcsr [{}]",
            in(reg) &mxcsr,
            options(nostack, preserves_flags)
        );
    }
}

/// Enable FTZ (Flush-To-Zero) mode on AArch64.
///
/// Sets the FZ bit in FPCR, causing denormal outputs to be flushed to zero.
///
/// # ARM vs x86 Differences
///
/// ARM FPCR.FZ only affects outputs (no universal DAZ equivalent).
/// Use `flush_denormal_f64/f32` in feedback paths for full coverage.
#[cfg(all(feature = "ftz-daz", target_arch = "aarch64"))]
pub fn enable_ftz_daz() {
    use std::arch::asm;

    const FZ_BIT: u64 = 1 << 24;

    unsafe {
        let mut fpcr: u64;
        asm!(
            "mrs {}, fpcr",
            out(reg) fpcr,
            options(nomem, nostack, preserves_flags)
        );
        fpcr |= FZ_BIT;
        asm!(
            "msr fpcr, {}",
            in(reg) fpcr,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// No-op stub for unsupported architectures when `ftz-daz` feature is enabled.
#[cfg(all(
    feature = "ftz-daz",
    not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))
))]
pub fn enable_ftz_daz() {
    // No-op on unsupported architectures
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

    #[cfg(all(feature = "ftz-daz", target_arch = "aarch64"))]
    #[test]
    fn test_enable_ftz_daz_sets_fz_bit() {
        use std::arch::asm;
        const FZ_BIT: u64 = 1 << 24;

        enable_ftz_daz();

        let fpcr: u64;
        unsafe {
            asm!(
                "mrs {}, fpcr",
                out(reg) fpcr,
                options(nomem, nostack, preserves_flags)
            );
        }
        assert_ne!(fpcr & FZ_BIT, 0, "FZ bit should be set after enable_ftz_daz()");
    }
}
