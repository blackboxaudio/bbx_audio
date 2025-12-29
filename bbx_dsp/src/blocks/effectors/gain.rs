//! Gain control block with dB input.

use std::marker::PhantomData;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::SmoothedValue,
};

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-60 to +30).
    pub level_db: Parameter<S>,

    /// Smoothed linear gain value for click-free parameter changes.
    gain_smoother: SmoothedValue,

    _phantom: PhantomData<S>,
}

impl<S: Sample> GainBlock<S> {
    /// Minimum gain in dB (silence threshold).
    const MIN_DB: f64 = -80.0;
    /// Maximum gain in dB.
    const MAX_DB: f64 = 30.0;

    /// Create a new `GainBlock` with the given level in dB.
    pub fn new(level_db: S) -> Self {
        let clamped_db = level_db.to_f64().clamp(Self::MIN_DB, Self::MAX_DB);
        let initial_gain = Self::db_to_linear(clamped_db);

        Self {
            level_db: Parameter::Constant(level_db),
            gain_smoother: SmoothedValue::new(initial_gain),
            _phantom: PhantomData,
        }
    }

    /// Create a unity gain (0 dB) block.
    pub fn unity() -> Self {
        Self::new(S::ZERO)
    }

    /// Convert dB to linear gain with range clamping.
    #[inline]
    fn db_to_linear(db: f64) -> f64 {
        let clamped = db.clamp(Self::MIN_DB, Self::MAX_DB);
        10.0_f64.powf(clamped / 20.0)
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        // Get target gain and set up smoothing
        let level_db = self.level_db.get_value(modulation_values).to_f64();
        let target_gain = Self::db_to_linear(level_db);

        // Only update smoother if target changed
        if (target_gain - self.gain_smoother.target()).abs() > 1e-10 {
            self.gain_smoother.set_target(target_gain, context.buffer_size);
        }

        for (ch, input) in inputs.iter().enumerate() {
            if ch >= outputs.len() {
                break;
            }

            // Reset smoother position for each channel (they all use same gain curve)
            let mut channel_smoother = self.gain_smoother.clone();

            for (i, &sample) in input.iter().enumerate() {
                if i >= outputs[ch].len() {
                    break;
                }
                let gain = channel_smoother.next_value();
                outputs[ch][i] = S::from_f64(sample.to_f64() * gain);
            }
        }

        // Advance the main smoother by buffer_size samples
        for _ in 0..context.buffer_size {
            self.gain_smoother.next_value();
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
