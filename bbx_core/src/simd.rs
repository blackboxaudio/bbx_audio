//! Portable SIMD utilities for audio DSP.
//!
//! This module provides SIMD-accelerated operations for common DSP tasks.
//! Requires the `simd` feature and nightly Rust.

use std::simd::{StdFloat, f32x4, f64x4};

use crate::sample::{SIMD_LANES, Sample};

pub const F32_LANES: usize = 4;
pub const F64_LANES: usize = 4;

#[inline]
pub fn fill_f32(slice: &mut [f32], value: f32) {
    let vec = f32x4::splat(value);
    let (chunks, remainder) = slice.as_chunks_mut::<F32_LANES>();

    for chunk in chunks {
        *chunk = vec.to_array();
    }
    remainder.fill(value);
}

#[inline]
pub fn fill_f64(slice: &mut [f64], value: f64) {
    let vec = f64x4::splat(value);
    let (chunks, remainder) = slice.as_chunks_mut::<F64_LANES>();

    for chunk in chunks {
        *chunk = vec.to_array();
    }
    remainder.fill(value);
}

#[inline]
pub fn apply_gain_f32(input: &[f32], output: &mut [f32], gain: f32) {
    debug_assert!(input.len() <= output.len());

    let gain_vec = f32x4::splat(gain);
    let len = input.len();
    let chunks = len / F32_LANES;
    let remainder_start = chunks * F32_LANES;

    for i in 0..chunks {
        let offset = i * F32_LANES;
        let in_chunk = f32x4::from_slice(&input[offset..]);
        let result = in_chunk * gain_vec;
        output[offset..offset + F32_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = input[i] * gain;
    }
}

#[inline]
pub fn apply_gain_f64(input: &[f64], output: &mut [f64], gain: f64) {
    debug_assert!(input.len() <= output.len());

    let gain_vec = f64x4::splat(gain);
    let len = input.len();
    let chunks = len / F64_LANES;
    let remainder_start = chunks * F64_LANES;

    for i in 0..chunks {
        let offset = i * F64_LANES;
        let in_chunk = f64x4::from_slice(&input[offset..]);
        let result = in_chunk * gain_vec;
        output[offset..offset + F64_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = input[i] * gain;
    }
}

#[inline]
pub fn multiply_add_f32(a: &[f32], b: &[f32], output: &mut [f32]) {
    debug_assert!(a.len() == b.len());
    debug_assert!(a.len() <= output.len());

    let len = a.len();
    let chunks = len / F32_LANES;
    let remainder_start = chunks * F32_LANES;

    for i in 0..chunks {
        let offset = i * F32_LANES;
        let a_chunk = f32x4::from_slice(&a[offset..]);
        let b_chunk = f32x4::from_slice(&b[offset..]);
        let result = a_chunk * b_chunk;
        output[offset..offset + F32_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = a[i] * b[i];
    }
}

#[inline]
pub fn multiply_add_f64(a: &[f64], b: &[f64], output: &mut [f64]) {
    debug_assert!(a.len() == b.len());
    debug_assert!(a.len() <= output.len());

    let len = a.len();
    let chunks = len / F64_LANES;
    let remainder_start = chunks * F64_LANES;

    for i in 0..chunks {
        let offset = i * F64_LANES;
        let a_chunk = f64x4::from_slice(&a[offset..]);
        let b_chunk = f64x4::from_slice(&b[offset..]);
        let result = a_chunk * b_chunk;
        output[offset..offset + F64_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = a[i] * b[i];
    }
}

pub fn sin_f32(input: &[f32], output: &mut [f32]) {
    debug_assert!(input.len() <= output.len());

    let len = input.len();
    let chunks = len / F32_LANES;
    let remainder_start = chunks * F32_LANES;

    for i in 0..chunks {
        let offset = i * F32_LANES;
        let in_chunk = f32x4::from_slice(&input[offset..]);
        let result = in_chunk.sin();
        output[offset..offset + F32_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = input[i].sin();
    }
}

pub fn sin_f64(input: &[f64], output: &mut [f64]) {
    debug_assert!(input.len() <= output.len());

    let len = input.len();
    let chunks = len / F64_LANES;
    let remainder_start = chunks * F64_LANES;

    for i in 0..chunks {
        let offset = i * F64_LANES;
        let in_chunk = f64x4::from_slice(&input[offset..]);
        let result = in_chunk.sin();
        output[offset..offset + F64_LANES].copy_from_slice(&result.to_array());
    }

    for i in remainder_start..len {
        output[i] = input[i].sin();
    }
}

// =============================================================================
// Generic SIMD operations using Sample trait
// =============================================================================

/// Fill a slice with a constant value using SIMD.
#[inline]
pub fn fill<S: Sample>(slice: &mut [S], value: S) {
    let vec = S::simd_splat(value);
    let chunks = slice.len() / SIMD_LANES;
    let remainder_start = chunks * SIMD_LANES;

    for i in 0..chunks {
        let offset = i * SIMD_LANES;
        slice[offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(vec));
    }
    slice[remainder_start..].fill(value);
}

/// Apply a gain value to an input slice and write to output using SIMD.
#[inline]
pub fn apply_gain<S: Sample>(input: &[S], output: &mut [S], gain: S)
where
    S::Simd: std::ops::Mul<Output = S::Simd>,
{
    debug_assert!(input.len() <= output.len());

    let gain_vec = S::simd_splat(gain);
    let len = input.len();
    let chunks = len / SIMD_LANES;
    let remainder_start = chunks * SIMD_LANES;

    for i in 0..chunks {
        let offset = i * SIMD_LANES;
        let in_chunk = S::simd_from_slice(&input[offset..]);
        let result = in_chunk * gain_vec;
        output[offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(result));
    }

    for i in remainder_start..len {
        output[i] = input[i] * gain;
    }
}

/// Element-wise multiply two slices and write to output using SIMD.
#[inline]
pub fn multiply_add<S: Sample>(a: &[S], b: &[S], output: &mut [S])
where
    S::Simd: std::ops::Mul<Output = S::Simd>,
{
    debug_assert!(a.len() == b.len());
    debug_assert!(a.len() <= output.len());

    let len = a.len();
    let chunks = len / SIMD_LANES;
    let remainder_start = chunks * SIMD_LANES;

    for i in 0..chunks {
        let offset = i * SIMD_LANES;
        let a_chunk = S::simd_from_slice(&a[offset..]);
        let b_chunk = S::simd_from_slice(&b[offset..]);
        let result = a_chunk * b_chunk;
        output[offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(result));
    }

    for i in remainder_start..len {
        output[i] = a[i] * b[i];
    }
}

/// Compute sine of each element using SIMD.
pub fn sin<S: Sample>(input: &[S], output: &mut [S]) {
    debug_assert!(input.len() <= output.len());

    let len = input.len();
    let chunks = len / SIMD_LANES;
    let remainder_start = chunks * SIMD_LANES;

    for i in 0..chunks {
        let offset = i * SIMD_LANES;
        let in_chunk = S::simd_from_slice(&input[offset..]);
        let result = in_chunk.sin();
        output[offset..offset + SIMD_LANES].copy_from_slice(&S::simd_to_array(result));
    }

    for i in remainder_start..len {
        output[i] = S::from_f64(input[i].to_f64().sin());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_f32() {
        let mut buffer = [0.0f32; 10];
        fill_f32(&mut buffer, 1.5);
        assert!(buffer.iter().all(|&x| x == 1.5));
    }

    #[test]
    fn test_fill_f64() {
        let mut buffer = [0.0f64; 10];
        fill_f64(&mut buffer, 2.5);
        assert!(buffer.iter().all(|&x| x == 2.5));
    }

    #[test]
    fn test_apply_gain_f32() {
        let input: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 10];
        apply_gain_f32(&input, &mut output, 0.5);

        for (i, &val) in output.iter().enumerate() {
            assert!((val - (i as f32) * 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_apply_gain_f64() {
        let input: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let mut output = vec![0.0f64; 10];
        apply_gain_f64(&input, &mut output, 0.5);

        for (i, &val) in output.iter().enumerate() {
            assert!((val - (i as f64) * 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_sin_f32() {
        let input: Vec<f32> = (0..10).map(|i| i as f32 * 0.1).collect();
        let mut output = vec![0.0f32; 10];
        sin_f32(&input, &mut output);

        for (i, &val) in output.iter().enumerate() {
            let expected = (i as f32 * 0.1).sin();
            assert!((val - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn test_sin_f64() {
        let input: Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
        let mut output = vec![0.0f64; 10];
        sin_f64(&input, &mut output);

        for (i, &val) in output.iter().enumerate() {
            let expected = (i as f64 * 0.1).sin();
            assert!((val - expected).abs() < 1e-10);
        }
    }
}
