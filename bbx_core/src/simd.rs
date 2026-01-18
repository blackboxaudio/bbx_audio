//! Portable SIMD utilities for audio DSP.
//!
//! This module provides SIMD-accelerated operations for common DSP tasks.
//! Requires the `simd` feature and nightly Rust.

use core::simd::{f32x4, f64x4};
use std::simd::StdFloat;

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
    S::Simd: core::ops::Mul<Output = S::Simd>,
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
    S::Simd: core::ops::Mul<Output = S::Simd>,
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

    // =============================================================================
    // Tests for typed f32/f64 SIMD functions
    // =============================================================================

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

    // =============================================================================
    // Edge case tests for non-aligned buffer sizes
    // =============================================================================

    #[test]
    fn test_fill_f32_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let mut buffer = vec![0.0f32; size];
            fill_f32(&mut buffer, 3.14);
            assert!(buffer.iter().all(|&x| x == 3.14), "Failed for size {}", size);
        }
    }

    #[test]
    fn test_fill_f64_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let mut buffer = vec![0.0f64; size];
            fill_f64(&mut buffer, 3.14);
            assert!(buffer.iter().all(|&x| x == 3.14), "Failed for size {}", size);
        }
    }

    #[test]
    fn test_apply_gain_f32_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let mut output = vec![0.0f32; size];
            apply_gain_f32(&input, &mut output, 2.0);
            for (i, &val) in output.iter().enumerate() {
                assert!((val - (i as f32) * 2.0).abs() < 1e-6, "Failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_apply_gain_f64_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input: Vec<f64> = (0..size).map(|i| i as f64).collect();
            let mut output = vec![0.0f64; size];
            apply_gain_f64(&input, &mut output, 2.0);
            for (i, &val) in output.iter().enumerate() {
                assert!((val - (i as f64) * 2.0).abs() < 1e-10, "Failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_sin_f32_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input: Vec<f32> = (0..size).map(|i| i as f32 * 0.1).collect();
            let mut output = vec![0.0f32; size];
            sin_f32(&input, &mut output);
            for (i, &val) in output.iter().enumerate() {
                let expected = (i as f32 * 0.1).sin();
                assert!((val - expected).abs() < 1e-5, "Failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_sin_f64_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input: Vec<f64> = (0..size).map(|i| i as f64 * 0.1).collect();
            let mut output = vec![0.0f64; size];
            sin_f64(&input, &mut output);
            for (i, &val) in output.iter().enumerate() {
                let expected = (i as f64 * 0.1).sin();
                assert!((val - expected).abs() < 1e-10, "Failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_multiply_add_f32() {
        let a: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..10).map(|i| (10 - i) as f32).collect();
        let mut output = vec![0.0f32; 10];
        multiply_add_f32(&a, &b, &mut output);

        for i in 0..10 {
            let expected = (i as f32) * ((10 - i) as f32);
            assert!((output[i] - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn test_multiply_add_f64() {
        let a: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..10).map(|i| (10 - i) as f64).collect();
        let mut output = vec![0.0f64; 10];
        multiply_add_f64(&a, &b, &mut output);

        for i in 0..10 {
            let expected = (i as f64) * ((10 - i) as f64);
            assert!((output[i] - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_multiply_add_f32_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let b: Vec<f32> = (0..size).map(|i| (i + 1) as f32).collect();
            let mut output = vec![0.0f32; size];
            multiply_add_f32(&a, &b, &mut output);
            for i in 0..size {
                let expected = (i as f32) * ((i + 1) as f32);
                assert!((output[i] - expected).abs() < 1e-6, "Failed for size {}", size);
            }
        }
    }

    // =============================================================================
    // Tests for generic Sample-based SIMD functions
    // =============================================================================

    #[test]
    fn test_generic_fill_f32() {
        let mut buffer = vec![0.0f32; 10];
        fill::<f32>(&mut buffer, 1.5);
        assert!(buffer.iter().all(|&x| x == 1.5));
    }

    #[test]
    fn test_generic_fill_f64() {
        let mut buffer = vec![0.0f64; 10];
        fill::<f64>(&mut buffer, 2.5);
        assert!(buffer.iter().all(|&x| x == 2.5));
    }

    #[test]
    fn test_generic_fill_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let mut buffer_f32 = vec![0.0f32; size];
            let mut buffer_f64 = vec![0.0f64; size];
            fill::<f32>(&mut buffer_f32, 3.14);
            fill::<f64>(&mut buffer_f64, 3.14);
            assert!(
                buffer_f32.iter().all(|&x| (x - 3.14).abs() < 1e-6),
                "f32 failed for size {}",
                size
            );
            assert!(
                buffer_f64.iter().all(|&x| (x - 3.14).abs() < 1e-10),
                "f64 failed for size {}",
                size
            );
        }
    }

    #[test]
    fn test_generic_apply_gain_f32() {
        let input: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 10];
        apply_gain::<f32>(&input, &mut output, 0.5);

        for (i, &val) in output.iter().enumerate() {
            assert!((val - (i as f32) * 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_generic_apply_gain_f64() {
        let input: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let mut output = vec![0.0f64; 10];
        apply_gain::<f64>(&input, &mut output, 0.5);

        for (i, &val) in output.iter().enumerate() {
            assert!((val - (i as f64) * 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_generic_apply_gain_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input_f32: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let input_f64: Vec<f64> = (0..size).map(|i| i as f64).collect();
            let mut output_f32 = vec![0.0f32; size];
            let mut output_f64 = vec![0.0f64; size];
            apply_gain::<f32>(&input_f32, &mut output_f32, 2.0);
            apply_gain::<f64>(&input_f64, &mut output_f64, 2.0);
            for (i, (&v32, &v64)) in output_f32.iter().zip(output_f64.iter()).enumerate() {
                assert!((v32 - (i as f32) * 2.0).abs() < 1e-6, "f32 failed for size {}", size);
                assert!((v64 - (i as f64) * 2.0).abs() < 1e-10, "f64 failed for size {}", size);
            }
        }
    }

    #[test]
    fn test_generic_multiply_add_f32() {
        let a: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..10).map(|i| (10 - i) as f32).collect();
        let mut output = vec![0.0f32; 10];
        multiply_add::<f32>(&a, &b, &mut output);

        for i in 0..10 {
            let expected = (i as f32) * ((10 - i) as f32);
            assert!((output[i] - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn test_generic_multiply_add_f64() {
        let a: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..10).map(|i| (10 - i) as f64).collect();
        let mut output = vec![0.0f64; 10];
        multiply_add::<f64>(&a, &b, &mut output);

        for i in 0..10 {
            let expected = (i as f64) * ((10 - i) as f64);
            assert!((output[i] - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_generic_sin_f32() {
        let input: Vec<f32> = (0..10).map(|i| i as f32 * 0.1).collect();
        let mut output = vec![0.0f32; 10];
        sin::<f32>(&input, &mut output);

        for (i, &val) in output.iter().enumerate() {
            let expected = (i as f32 * 0.1).sin();
            assert!((val - expected).abs() < 1e-5);
        }
    }

    #[test]
    fn test_generic_sin_f64() {
        let input: Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
        let mut output = vec![0.0f64; 10];
        sin::<f64>(&input, &mut output);

        for (i, &val) in output.iter().enumerate() {
            let expected = (i as f64 * 0.1).sin();
            assert!((val - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_generic_sin_edge_sizes() {
        for size in [0, 1, 2, 3, 5, 7, 9, 15] {
            let input_f32: Vec<f32> = (0..size).map(|i| i as f32 * 0.1).collect();
            let input_f64: Vec<f64> = (0..size).map(|i| i as f64 * 0.1).collect();
            let mut output_f32 = vec![0.0f32; size];
            let mut output_f64 = vec![0.0f64; size];
            sin::<f32>(&input_f32, &mut output_f32);
            sin::<f64>(&input_f64, &mut output_f64);
            for i in 0..size {
                let expected_f32 = (i as f32 * 0.1).sin();
                let expected_f64 = (i as f64 * 0.1).sin();
                assert!(
                    (output_f32[i] - expected_f32).abs() < 1e-5,
                    "f32 failed for size {}",
                    size
                );
                assert!(
                    (output_f64[i] - expected_f64).abs() < 1e-10,
                    "f64 failed for size {}",
                    size
                );
            }
        }
    }
}
