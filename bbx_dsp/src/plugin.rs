//! Plugin DSP trait for FFI integration.
//!
//! This module defines the `PluginDsp` trait that consumers implement
//! to define their plugin's DSP processing chain.

use crate::context::DspContext;

/// Trait for plugin-specific DSP implementations.
///
/// Consumers implement this trait to define their audio processing chain.
/// The FFI layer uses this trait to manage the DSP lifecycle.
///
/// # Example
///
/// ```ignore
/// use bbx_dsp::{PluginDsp, context::DspContext};
/// use bbx_dsp::blocks::effectors::gain::GainBlock;
///
/// pub struct PluginGraph {
///     pub gain: GainBlock<f32>,
/// }
///
/// impl PluginDsp for PluginGraph {
///     fn new() -> Self {
///         Self { gain: GainBlock::new(0.0) }
///     }
///
///     fn prepare(&mut self, context: &DspContext) {
///         // Initialize blocks for the given sample rate/buffer size
///     }
///
///     fn reset(&mut self) {
///         // Clear filter states, etc.
///     }
///
///     fn apply_parameters(&mut self, params: &[f32]) {
///         // Map parameter array to block fields
///     }
///
///     fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], context: &DspContext) {
///         // Process audio through the chain
///     }
/// }
/// ```
pub trait PluginDsp: Default + Send + 'static {
    /// Create a new instance with default configuration.
    fn new() -> Self;

    /// Prepare the DSP chain for playback.
    ///
    /// Called when audio specs change (sample rate, buffer size, channels).
    fn prepare(&mut self, context: &DspContext);

    /// Reset all DSP state.
    ///
    /// Called to clear filter histories, oscillator phases, etc.
    fn reset(&mut self);

    /// Apply parameter values from a flat array.
    ///
    /// The parameter indices are plugin-specific, typically defined
    /// via generated constants from `parameters.json`.
    fn apply_parameters(&mut self, params: &[f32]);

    /// Process a block of audio.
    ///
    /// - `inputs`: Array of input channel buffers
    /// - `outputs`: Array of output channel buffers
    /// - `context`: DSP context with sample rate, buffer size, etc.
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], context: &DspContext);
}
