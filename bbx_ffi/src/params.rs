//! Parameter index constants for FFI parameter passing.
//!
//! These constants define the indices into the flat parameter array
//! passed from C++ to Rust during audio processing.
//!
//! Parameters match those defined in `template-plugin/parameters.json`.

/// Invert left channel phase (0.0 = off, 1.0 = on).
pub const PARAM_INVERT_LEFT: usize = 0;

/// Invert right channel phase (0.0 = off, 1.0 = on).
pub const PARAM_INVERT_RIGHT: usize = 1;

/// Channel routing mode (0 = Stereo, 1 = Left, 2 = Right, 3 = Swap).
pub const PARAM_CHANNEL_MODE: usize = 2;

/// Sum to mono (0.0 = off, 1.0 = on).
pub const PARAM_MONO: usize = 3;

/// Gain level in dB (-60 to +30).
pub const PARAM_GAIN: usize = 4;

/// Pan position (-100 to +100).
pub const PARAM_PAN: usize = 5;

/// DC offset removal enabled (0.0 = off, 1.0 = on).
pub const PARAM_DC_OFFSET: usize = 6;

/// Total number of parameters.
pub const PARAM_COUNT: usize = 7;

/// Default parameter values loaded from parameters.json at build time.
#[derive(Debug, Clone)]
pub struct ParamDefaults {
    pub invert_left: bool,
    pub invert_right: bool,
    pub channel_mode: i32,
    pub mono: bool,
    pub gain_db: f32,
    pub pan: f32,
    pub dc_offset: bool,
}

/// Get the default parameter values.
///
/// These are generated at build time from `template-plugin/parameters.json`.
pub fn default_params() -> ParamDefaults {
    include!(concat!(env!("OUT_DIR"), "/param_defaults.rs"))
}
