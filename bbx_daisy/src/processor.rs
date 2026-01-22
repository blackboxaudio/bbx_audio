//! Audio processor trait for embedded DSP.
//!
//! This module provides the [`AudioProcessor`] trait which enables safe,
//! stateful audio processing without user-written `unsafe` code.

use crate::{FrameBuffer, audio::BLOCK_SIZE, controls::Controls};

/// Trait for audio processing on Daisy hardware.
///
/// Implement this trait to create audio processors with encapsulated state.
/// The macro [`bbx_daisy_audio!`](crate::bbx_daisy_audio) handles all the
/// unsafe static state management internally.
///
/// # Example
///
/// ```ignore
/// use bbx_daisy::prelude::*;
///
/// struct TunableSine {
///     phase: f32,
/// }
///
/// impl AudioProcessor for TunableSine {
///     fn process(
///         &mut self,
///         _input: &FrameBuffer<BLOCK_SIZE>,
///         output: &mut FrameBuffer<BLOCK_SIZE>,
///         controls: &Controls,
///     ) {
///         // Map knob1 to frequency (110Hz - 880Hz)
///         let frequency = 110.0 + controls.knob1 * 770.0;
///         let phase_inc = frequency / DEFAULT_SAMPLE_RATE;
///
///         for i in 0..BLOCK_SIZE {
///             let sample = sinf(self.phase * 2.0 * PI) * 0.5;
///             output.set_frame(i, sample, sample);
///             self.phase += phase_inc;
///             if self.phase >= 1.0 {
///                 self.phase -= 1.0;
///             }
///         }
///     }
/// }
///
/// bbx_daisy_audio!(TunableSine, TunableSine { phase: 0.0 });
/// ```
pub trait AudioProcessor: 'static {
    /// Process a block of audio samples.
    ///
    /// Called from the DMA interrupt at audio rate. Must complete within
    /// the buffer period (~0.67ms at 48kHz with 32-sample blocks).
    ///
    /// # Arguments
    ///
    /// * `input` - Input samples from the codec ADC
    /// * `output` - Output buffer to fill for the codec DAC
    /// * `controls` - Current values of hardware controls (knobs, CVs)
    ///
    /// # Realtime Safety
    ///
    /// This method runs in interrupt context and must complete within
    /// the buffer period. Avoid:
    /// - Heap allocations
    /// - Blocking operations
    /// - Unbounded loops
    fn process(&mut self, input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>, controls: &Controls);

    /// Prepare the processor for a given sample rate.
    ///
    /// Called once before audio processing begins. Use this to recalculate
    /// any sample-rate-dependent coefficients.
    fn prepare(&mut self, _sample_rate: f32) {}

    /// Reset the processor state.
    ///
    /// Called to clear internal state (filters, delay lines, etc.).
    fn reset(&mut self) {}
}
