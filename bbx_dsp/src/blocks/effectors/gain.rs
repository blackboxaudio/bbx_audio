//! Gain control block with dB input.

use std::marker::PhantomData;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-60 to +30).
    pub level_db: Parameter<S>,

    /// Smoothed linear gain value for click-free parameter changes.
    gain_smoother: LinearSmoothedValue<S>,

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
            gain_smoother: LinearSmoothedValue::new(S::from_f64(initial_gain)),
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
        let target_gain = S::from_f64(Self::db_to_linear(level_db));

        // Only update smoother if target changed significantly
        let current_target = self.gain_smoother.target();
        if (target_gain.to_f64() - current_target.to_f64()).abs() > 1e-9 {
            self.gain_smoother.set_target_value(target_gain);
        }

        let num_channels = inputs.len().min(outputs.len());

        // Fast path: constant gain when not smoothing
        if !self.gain_smoother.is_smoothing() {
            let gain = self.gain_smoother.current().to_f64();
            for ch in 0..num_channels {
                let len = inputs[ch].len().min(outputs[ch].len());
                for i in 0..len {
                    outputs[ch][i] = S::from_f64(inputs[ch][i].to_f64() * gain);
                }
            }
            return;
        }

        // Smoothing path: clone smoother per channel for identical gain curves
        for ch in 0..num_channels {
            let len = inputs[ch].len().min(outputs[ch].len());
            let mut channel_smoother = self.gain_smoother.clone();

            for i in 0..len {
                let gain = channel_smoother.get_next_value();
                outputs[ch][i] = S::from_f64(inputs[ch][i].to_f64() * gain.to_f64());
            }
        }

        // Advance the main smoother by buffer_size samples
        self.gain_smoother.skip(context.buffer_size as i32);
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
