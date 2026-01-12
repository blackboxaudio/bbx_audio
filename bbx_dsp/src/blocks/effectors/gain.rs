//! Gain control block with dB input.

#[cfg(feature = "simd")]
use bbx_core::simd::apply_gain;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// Maximum buffer size for stack-allocated smoothing cache.
const MAX_BUFFER_SIZE: usize = 4096;

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-80 to +30).
    pub level_db: Parameter<S>,

    /// Base gain multiplier (linear) applied statically to the signal.
    pub base_gain: S,
}

impl<S: Sample> GainBlock<S> {
    /// Minimum gain in dB (silence threshold).
    const MIN_DB: f64 = -80.0;
    /// Maximum gain in dB.
    const MAX_DB: f64 = 30.0;

    /// Create a new `GainBlock` with the given level in dB and an optional base gain multiplier.
    pub fn new(level_db: S, base_gain: Option<S>) -> Self {
        let clamped_db = level_db.to_f64().clamp(Self::MIN_DB, Self::MAX_DB);
        let initial_gain = Self::db_to_linear(clamped_db);

        let mut param = Parameter::constant(level_db);
        param.set_immediate(S::from_f64(initial_gain));

        Self {
            level_db: param,
            base_gain: base_gain.unwrap_or(S::ONE),
        }
    }

    /// Create a unity gain (0 dB) block.
    pub fn unity() -> Self {
        Self::new(S::ZERO, None)
    }

    /// Convert dB to linear gain with range clamping.
    #[inline]
    fn db_to_linear(db: f64) -> f64 {
        let clamped = db.clamp(Self::MIN_DB, Self::MAX_DB);
        10.0_f64.powf(clamped / 20.0)
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn prepare(&mut self, context: &DspContext) {
        self.level_db.prepare(context.sample_rate);
    }

    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        // Get target gain from dB source and convert to linear
        let level_db = self.level_db.get_raw_value(modulation_values).to_f64();
        let target_gain = S::from_f64(Self::db_to_linear(level_db));

        // Set the linear target directly (smoothing happens in linear space)
        self.level_db.set_target(target_gain);

        let num_channels = inputs.len().min(outputs.len());

        // Fast path: constant gain when not smoothing
        if !self.level_db.is_smoothing() {
            let gain = self.level_db.current() * self.base_gain;

            #[cfg(feature = "simd")]
            {
                for ch in 0..num_channels {
                    let len = inputs[ch].len().min(outputs[ch].len());
                    apply_gain(&inputs[ch][..len], &mut outputs[ch][..len], gain);
                }
                return;
            }

            #[cfg(not(feature = "simd"))]
            {
                for ch in 0..num_channels {
                    let len = inputs[ch].len().min(outputs[ch].len());
                    for i in 0..len {
                        outputs[ch][i] = inputs[ch][i] * gain;
                    }
                }
                return;
            }
        }

        // Smoothing path: compute smoothed values once, apply to all channels
        let len = inputs.first().map_or(0, |ch| ch.len().min(context.buffer_size));
        debug_assert!(len <= MAX_BUFFER_SIZE, "buffer_size exceeds MAX_BUFFER_SIZE");

        // Pre-compute all smoothed gain values into a stack buffer
        let mut gain_values: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        for gain_value in gain_values.iter_mut().take(len) {
            *gain_value = self.level_db.next_value() * self.base_gain;
        }

        // Apply the same gain curve to all channels
        for ch in 0..num_channels {
            let ch_len = inputs[ch].len().min(outputs[ch].len()).min(len);
            for (i, &gain) in gain_values.iter().enumerate().take(ch_len) {
                outputs[ch][i] = inputs[ch][i] * gain;
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_EFFECTOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_EFFECTOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}
