//! Parameter index constants for FFI parameter passing.
//!
//! These constants define the indices into the flat parameter array
//! passed from C++ to Rust during audio processing.

/// Oscillator base frequency in Hz.
pub const PARAM_OSC_FREQUENCY: usize = 0;

/// Oscillator pitch offset in semitones.
pub const PARAM_OSC_PITCH_OFFSET: usize = 1;

/// Envelope attack time in seconds.
pub const PARAM_ENV_ATTACK: usize = 2;

/// Envelope decay time in seconds.
pub const PARAM_ENV_DECAY: usize = 3;

/// Envelope sustain level (0.0 to 1.0).
pub const PARAM_ENV_SUSTAIN: usize = 4;

/// Envelope release time in seconds.
pub const PARAM_ENV_RELEASE: usize = 5;

/// LFO frequency in Hz.
pub const PARAM_LFO_FREQUENCY: usize = 6;

/// LFO depth (0.0 to 1.0).
pub const PARAM_LFO_DEPTH: usize = 7;

/// Overdrive drive amount.
pub const PARAM_DRIVE: usize = 8;

/// Output level (0.0 to 1.0).
pub const PARAM_LEVEL: usize = 9;

/// Total number of parameters.
pub const PARAM_COUNT: usize = 10;
