//! Embedded DSP processing context.
//!
//! This module provides [`EmbeddedDspContext`], a memory-optimized version
//! of `bbx_dsp::DspContext` designed for embedded targets.

/// Default buffer size for embedded DSP (32 samples for low latency).
pub const DEFAULT_BUFFER_SIZE: usize = 32;

/// Default sample rate for Daisy hardware (48000 Hz).
pub const DEFAULT_SAMPLE_RATE: f32 = 48000.0;

/// Runtime context for embedded DSP processing.
///
/// A simplified version of `bbx_dsp::DspContext` optimized for embedded targets:
///
/// - Uses `f32` for sample rate (saves 4 bytes vs `f64`)
/// - Uses `u32` for sample counter (sufficient for ~24 hours at 48kHz)
/// - Buffer size is a const generic (known at compile time)
///
/// # Type Parameters
///
/// - `BUFFER_SIZE`: Number of samples per processing block (typically 32 or 64)
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::EmbeddedDspContext;
///
/// let mut ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);
/// assert_eq!(ctx.buffer_size(), 32);
/// assert_eq!(ctx.sample_rate, 48000.0);
///
/// ctx.advance();
/// assert_eq!(ctx.current_sample, 32);
/// ```
#[derive(Clone, Copy)]
pub struct EmbeddedDspContext<const BUFFER_SIZE: usize = DEFAULT_BUFFER_SIZE> {
    /// The audio sample rate in Hz (typically 48000.0 for Daisy).
    pub sample_rate: f32,

    /// The number of output channels.
    pub num_channels: usize,

    /// The absolute sample position since processing started.
    ///
    /// At 48kHz, a `u32` can count ~24 hours before wrapping.
    pub current_sample: u32,
}

impl<const BUFFER_SIZE: usize> EmbeddedDspContext<BUFFER_SIZE> {
    /// Create a new context with the given sample rate.
    ///
    /// Defaults to stereo output (2 channels).
    #[inline]
    pub const fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            num_channels: 2,
            current_sample: 0,
        }
    }

    /// Create a new context with the given sample rate and channel count.
    #[inline]
    pub const fn with_channels(sample_rate: f32, num_channels: usize) -> Self {
        Self {
            sample_rate,
            num_channels,
            current_sample: 0,
        }
    }

    /// Get the buffer size (const generic parameter).
    #[inline]
    pub const fn buffer_size(&self) -> usize {
        BUFFER_SIZE
    }

    /// Advance the sample counter by one buffer's worth of samples.
    ///
    /// Call this at the end of each processing cycle.
    #[inline]
    pub fn advance(&mut self) {
        self.current_sample = self.current_sample.wrapping_add(BUFFER_SIZE as u32);
    }

    /// Reset the sample counter to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.current_sample = 0;
    }

    /// Get the current time in seconds since processing started.
    #[inline]
    pub fn current_time_secs(&self) -> f32 {
        self.current_sample as f32 / self.sample_rate
    }

    /// Get the duration of one sample in seconds.
    #[inline]
    pub fn sample_period(&self) -> f32 {
        1.0 / self.sample_rate
    }

    /// Get the duration of one buffer in seconds.
    #[inline]
    pub fn buffer_period(&self) -> f32 {
        BUFFER_SIZE as f32 / self.sample_rate
    }
}

impl<const BUFFER_SIZE: usize> Default for EmbeddedDspContext<BUFFER_SIZE> {
    #[inline]
    fn default() -> Self {
        Self::new(DEFAULT_SAMPLE_RATE)
    }
}

impl<const BUFFER_SIZE: usize> From<EmbeddedDspContext<BUFFER_SIZE>> for bbx_dsp::context::DspContext {
    fn from(ctx: EmbeddedDspContext<BUFFER_SIZE>) -> Self {
        Self {
            sample_rate: ctx.sample_rate as f64,
            num_channels: ctx.num_channels,
            buffer_size: BUFFER_SIZE,
            current_sample: ctx.current_sample as u64,
            channel_layout: bbx_dsp::ChannelLayout::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_new() {
        let ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);
        assert_eq!(ctx.sample_rate, 48000.0);
        assert_eq!(ctx.num_channels, 2);
        assert_eq!(ctx.current_sample, 0);
        assert_eq!(ctx.buffer_size(), 32);
    }

    #[test]
    fn context_with_channels() {
        let ctx: EmbeddedDspContext<64> = EmbeddedDspContext::with_channels(44100.0, 4);
        assert_eq!(ctx.sample_rate, 44100.0);
        assert_eq!(ctx.num_channels, 4);
        assert_eq!(ctx.buffer_size(), 64);
    }

    #[test]
    fn context_advance() {
        let mut ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);
        assert_eq!(ctx.current_sample, 0);

        ctx.advance();
        assert_eq!(ctx.current_sample, 32);

        ctx.advance();
        assert_eq!(ctx.current_sample, 64);
    }

    #[test]
    fn context_reset() {
        let mut ctx: EmbeddedDspContext<32> = EmbeddedDspContext::new(48000.0);
        ctx.advance();
        ctx.advance();
        assert_eq!(ctx.current_sample, 64);

        ctx.reset();
        assert_eq!(ctx.current_sample, 0);
    }

    #[test]
    fn context_time_calculations() {
        let ctx: EmbeddedDspContext<48000> = EmbeddedDspContext::new(48000.0);
        let period = ctx.sample_period();
        assert!((period - (1.0 / 48000.0)).abs() < 1e-9);

        let buffer_period = ctx.buffer_period();
        assert!((buffer_period - 1.0).abs() < 1e-6);
    }

    #[test]
    fn context_wrapping_add() {
        let mut ctx: EmbeddedDspContext<1> = EmbeddedDspContext::new(48000.0);
        ctx.current_sample = u32::MAX;
        ctx.advance();
        assert_eq!(ctx.current_sample, 0);
    }
}
