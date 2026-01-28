//! Stack-allocated buffer types for embedded DSP processing.
//!
//! This module provides buffer types designed for embedded targets where
//! heap allocation is unavailable:
//!
//! - [`StaticSampleBuffer`] - Per-channel sample buffer for DSP processing
//! - [`FrameBuffer`] - Interleaved multi-channel buffer for hardware I/O

use core::ops::{Index, IndexMut};

use bbx_core::{Buffer, Sample};

/// Stack-allocated sample buffer for embedded DSP processing.
///
/// Unlike `SampleBuffer` (which uses `Vec`), this buffer has a compile-time
/// fixed size and lives entirely on the stack. Use for embedded targets
/// where heap allocation is unavailable.
///
/// # Type Parameters
///
/// - `N`: Buffer size in samples (typically 32 or 64 for low-latency embedded)
/// - `S`: Sample type (`f32` or `f64`)
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::StaticSampleBuffer;
///
/// let mut buffer: StaticSampleBuffer<64, f32> = StaticSampleBuffer::new();
/// buffer.fill(0.5);
/// assert_eq!(buffer[0], 0.5);
/// ```
#[derive(Clone, Copy)]
pub struct StaticSampleBuffer<const N: usize, S: Sample = f32> {
    data: [S; N],
}

impl<const N: usize, S: Sample> StaticSampleBuffer<N, S> {
    /// Create a new buffer filled with zeros.
    #[inline]
    pub const fn new() -> Self {
        Self { data: [S::ZERO; N] }
    }

    /// Fill the buffer with a constant value.
    #[inline]
    pub fn fill(&mut self, value: S) {
        self.data.fill(value);
    }

    /// Copy samples from a slice into this buffer.
    ///
    /// Copies up to `min(source.len(), N)` samples.
    #[inline]
    pub fn copy_from_slice(&mut self, source: &[S]) {
        let len = source.len().min(N);
        self.data[..len].copy_from_slice(&source[..len]);
    }

    /// Copy samples from this buffer into a target slice.
    ///
    /// Copies up to `min(target.len(), N)` samples.
    #[inline]
    pub fn copy_to_slice(&self, target: &mut [S]) {
        let len = target.len().min(N);
        target[..len].copy_from_slice(&self.data[..len]);
    }

    /// Get the buffer capacity (always equal to N).
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize, S: Sample> Default for StaticSampleBuffer<N, S> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, S: Sample> Buffer<S> for StaticSampleBuffer<N, S> {
    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn is_empty(&self) -> bool {
        N == 0
    }

    #[inline]
    fn as_slice(&self) -> &[S] {
        &self.data
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [S] {
        &mut self.data
    }

    #[inline]
    fn clear(&mut self) {
        self.zeroize();
    }

    #[inline]
    fn zeroize(&mut self) {
        self.data.fill(S::ZERO);
    }
}

impl<const N: usize, S: Sample> Index<usize> for StaticSampleBuffer<N, S> {
    type Output = S;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize, S: Sample> IndexMut<usize> for StaticSampleBuffer<N, S> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

/// Interleaved multi-channel buffer for SAI/DMA hardware output.
///
/// Format: `[[L0, R0], [L1, R1], [L2, R2], ..., [LN, RN]]`
///
/// Required by Daisy codecs (PCM3060, WM8731, AK4556) for DMA transfers.
/// Only used at the hardware boundary—internal DSP processing uses
/// separate per-channel buffers.
///
/// # Type Parameters
///
/// - `N`: Number of frames (sample pairs)
/// - `C`: Channels per frame (default: 2 for stereo)
/// - `S`: Sample type (`f32` or `f64`)
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::FrameBuffer;
///
/// let mut buffer: FrameBuffer<64, 2, f32> = FrameBuffer::new();
/// buffer.set_frame(0, 0.5, -0.5);
/// assert_eq!(buffer.frame(0), &[0.5, -0.5]);
/// ```
#[derive(Clone, Copy)]
pub struct FrameBuffer<const N: usize, const C: usize = 2, S: Sample = f32> {
    data: [[S; C]; N],
}

impl<const N: usize, const C: usize, S: Sample> FrameBuffer<N, C, S> {
    /// Create a new buffer filled with zeros.
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [[S::ZERO; C]; N],
        }
    }

    /// Get the number of frames in the buffer.
    #[inline]
    pub const fn frame_count(&self) -> usize {
        N
    }

    /// Get the number of channels per frame.
    #[inline]
    pub const fn channel_count(&self) -> usize {
        C
    }

    /// Get a reference to a frame by index.
    #[inline]
    pub fn frame(&self, index: usize) -> &[S; C] {
        &self.data[index]
    }

    /// Get a mutable reference to a frame by index.
    #[inline]
    pub fn frame_mut(&mut self, index: usize) -> &mut [S; C] {
        &mut self.data[index]
    }

    /// Set a stereo frame (convenience method for C=2).
    #[inline]
    pub fn set_frame(&mut self, index: usize, left: S, right: S)
    where
        [S; C]: AsRef<[S]>,
    {
        if C >= 2 {
            self.data[index][0] = left;
            self.data[index][1] = right;
        }
    }

    /// Fill all frames with zeros.
    #[inline]
    pub fn zeroize(&mut self) {
        for frame in &mut self.data {
            frame.fill(S::ZERO);
        }
    }

    /// Get the total number of samples (frames * channels).
    #[inline]
    pub const fn total_samples(&self) -> usize {
        N * C
    }
}

impl<const N: usize, const C: usize, S: Sample> Default for FrameBuffer<N, C, S> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, S: Sample> FrameBuffer<N, 2, S> {
    /// Copy from separate left/right channel buffers into interleaved format.
    ///
    /// Copies up to `min(left.len(), right.len(), N)` frames.
    #[inline]
    pub fn interleave_from(&mut self, left: &[S], right: &[S]) {
        let len = left.len().min(right.len()).min(N);
        for i in 0..len {
            self.data[i][0] = left[i];
            self.data[i][1] = right[i];
        }
    }

    /// Copy interleaved frames into separate left/right channel buffers.
    ///
    /// Copies up to `min(left.len(), right.len(), N)` frames.
    #[inline]
    pub fn deinterleave_to(&self, left: &mut [S], right: &mut [S]) {
        let len = left.len().min(right.len()).min(N);
        for i in 0..len {
            left[i] = self.data[i][0];
            right[i] = self.data[i][1];
        }
    }
}

impl<const N: usize, const C: usize, S: Sample> Index<usize> for FrameBuffer<N, C, S> {
    type Output = [S; C];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize, const C: usize, S: Sample> IndexMut<usize> for FrameBuffer<N, C, S> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_sample_buffer_new_is_zeroed() {
        let buffer: StaticSampleBuffer<64, f32> = StaticSampleBuffer::new();
        assert_eq!(buffer.len(), 64);
        for i in 0..64 {
            assert_eq!(buffer[i], 0.0);
        }
    }

    #[test]
    fn static_sample_buffer_fill() {
        let mut buffer: StaticSampleBuffer<32, f32> = StaticSampleBuffer::new();
        buffer.fill(0.5);
        for i in 0..32 {
            assert_eq!(buffer[i], 0.5);
        }
    }

    #[test]
    fn static_sample_buffer_copy_from_slice() {
        let mut buffer: StaticSampleBuffer<8, f32> = StaticSampleBuffer::new();
        let source = [1.0, 2.0, 3.0, 4.0];
        buffer.copy_from_slice(&source);
        assert_eq!(buffer[0], 1.0);
        assert_eq!(buffer[1], 2.0);
        assert_eq!(buffer[2], 3.0);
        assert_eq!(buffer[3], 4.0);
        assert_eq!(buffer[4], 0.0);
    }

    #[test]
    fn static_sample_buffer_implements_buffer_trait() {
        let mut buffer: StaticSampleBuffer<16, f32> = StaticSampleBuffer::new();
        assert_eq!(buffer.len(), 16);
        assert!(!buffer.is_empty());

        buffer.fill(1.0);
        assert_eq!(buffer.as_slice()[0], 1.0);

        buffer.zeroize();
        assert_eq!(buffer.as_slice()[0], 0.0);

        buffer.as_mut_slice()[5] = 0.75;
        assert_eq!(buffer[5], 0.75);
    }

    #[test]
    fn frame_buffer_new_is_zeroed() {
        let buffer: FrameBuffer<32, 2, f32> = FrameBuffer::new();
        assert_eq!(buffer.frame_count(), 32);
        assert_eq!(buffer.channel_count(), 2);
        for i in 0..32 {
            assert_eq!(buffer.frame(i), &[0.0, 0.0]);
        }
    }

    #[test]
    fn frame_buffer_set_frame() {
        let mut buffer: FrameBuffer<8, 2, f32> = FrameBuffer::new();
        buffer.set_frame(0, 0.5, -0.5);
        buffer.set_frame(1, 0.75, -0.75);
        assert_eq!(buffer[0], [0.5, -0.5]);
        assert_eq!(buffer[1], [0.75, -0.75]);
    }

    #[test]
    fn frame_buffer_interleave_deinterleave() {
        let left = [0.1_f32, 0.2, 0.3, 0.4];
        let right = [0.5_f32, 0.6, 0.7, 0.8];

        let mut buffer: FrameBuffer<4, 2, f32> = FrameBuffer::new();
        buffer.interleave_from(&left, &right);

        assert_eq!(buffer[0], [0.1, 0.5]);
        assert_eq!(buffer[1], [0.2, 0.6]);
        assert_eq!(buffer[2], [0.3, 0.7]);
        assert_eq!(buffer[3], [0.4, 0.8]);

        let mut out_left = [0.0_f32; 4];
        let mut out_right = [0.0_f32; 4];
        buffer.deinterleave_to(&mut out_left, &mut out_right);

        assert_eq!(out_left, left);
        assert_eq!(out_right, right);
    }

    #[test]
    fn frame_buffer_total_samples() {
        let buffer: FrameBuffer<64, 2, f32> = FrameBuffer::new();
        assert_eq!(buffer.total_samples(), 128);
    }
}
