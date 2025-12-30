//! DSP processing context.
//!
//! This module provides [`DspContext`], which carries audio processing parameters
//! through the DSP graph during block processing.

/// Default buffer size for DSP graphs (512 samples).
pub const DEFAULT_BUFFER_SIZE: usize = 512;

/// Default sample rate for DSP graphs (44100 Hz).
pub const DEFAULT_SAMPLE_RATE: f64 = 44100.0;

/// Runtime context passed to blocks during audio processing.
///
/// Contains the audio specification (sample rate, channels, buffer size) and
/// the current playback position. This context is passed to every block's
/// `process()` method, allowing time-dependent calculations.
#[derive(Clone)]
pub struct DspContext {
    /// The audio sample rate in Hz (e.g., 44100.0, 48000.0).
    pub sample_rate: f64,

    /// The number of output channels (e.g., 2 for stereo).
    pub num_channels: usize,

    /// The number of samples processed per block.
    pub buffer_size: usize,

    /// The absolute sample position since playback started.
    /// Increments by `buffer_size` after each process cycle.
    pub current_sample: u64,
}
