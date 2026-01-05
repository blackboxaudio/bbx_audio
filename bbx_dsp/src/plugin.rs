//! Plugin DSP trait for FFI integration.
//!
//! This module defines the `PluginDsp` trait that consumers implement
//! to define their plugin's DSP processing chain.

use bbx_midi::MidiEvent;

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
///     fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], midi_events: &[MidiEvent], context: &DspContext) {
///         // Process audio and MIDI through the chain
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

    /// Process a block of audio with MIDI events.
    ///
    /// - `inputs`: Array of input channel buffers
    /// - `outputs`: Array of output channel buffers
    /// - `midi_events`: MIDI events with sample-accurate timing (sorted by sample_offset)
    /// - `context`: DSP context with sample rate, buffer size, etc.
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        midi_events: &[MidiEvent],
        context: &DspContext,
    );

    /// Called when a note-on event is received.
    ///
    /// Default implementation does nothing (suitable for effect plugins).
    #[allow(unused_variables)]
    fn note_on(&mut self, note: u8, velocity: u8, sample_offset: u32) {}

    /// Called when a note-off event is received.
    ///
    /// Default implementation does nothing (suitable for effect plugins).
    #[allow(unused_variables)]
    fn note_off(&mut self, note: u8, sample_offset: u32) {}

    /// Called when a control change event is received.
    ///
    /// Default implementation does nothing.
    #[allow(unused_variables)]
    fn control_change(&mut self, cc: u8, value: u8, sample_offset: u32) {}

    /// Called when a pitch bend event is received.
    ///
    /// Default implementation does nothing.
    #[allow(unused_variables)]
    fn pitch_bend(&mut self, value: i16, sample_offset: u32) {}
}
