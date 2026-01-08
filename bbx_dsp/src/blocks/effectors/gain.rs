//! Gain control block with dB input.

#[cfg(feature = "simd")]
use crate::buffer::{apply_gain_f32, apply_gain_f64};
use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// Maximum buffer size for stack-allocated smoothing cache.
const MAX_BUFFER_SIZE: usize = 4096;

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-60 to +30).
    pub level_db: Parameter<S>,

    /// Smoothed linear gain value for click-free parameter changes.
    gain_smoother: LinearSmoothedValue<S>,
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

            #[cfg(feature = "simd")]
            {
                use std::any::TypeId;

                if TypeId::of::<S>() == TypeId::of::<f32>() {
                    let gain_f32 = gain as f32;
                    for ch in 0..num_channels {
                        let len = inputs[ch].len().min(outputs[ch].len());
                        let input_f32 = unsafe { std::slice::from_raw_parts(inputs[ch].as_ptr() as *const f32, len) };
                        let output_f32 =
                            unsafe { std::slice::from_raw_parts_mut(outputs[ch].as_mut_ptr() as *mut f32, len) };
                        apply_gain_f32(input_f32, output_f32, gain_f32);
                    }
                } else if TypeId::of::<S>() == TypeId::of::<f64>() {
                    for ch in 0..num_channels {
                        let len = inputs[ch].len().min(outputs[ch].len());
                        let input_f64 = unsafe { std::slice::from_raw_parts(inputs[ch].as_ptr() as *const f64, len) };
                        let output_f64 =
                            unsafe { std::slice::from_raw_parts_mut(outputs[ch].as_mut_ptr() as *mut f64, len) };
                        apply_gain_f64(input_f64, output_f64, gain);
                    }
                }
                return;
            }

            #[cfg(not(feature = "simd"))]
            {
                for ch in 0..num_channels {
                    let len = inputs[ch].len().min(outputs[ch].len());
                    for i in 0..len {
                        outputs[ch][i] = S::from_f64(inputs[ch][i].to_f64() * gain);
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
            *gain_value = self.gain_smoother.get_next_value();
        }

        // Apply the same gain curve to all channels
        for ch in 0..num_channels {
            let ch_len = inputs[ch].len().min(outputs[ch].len()).min(len);
            for (i, &gain) in gain_values.iter().enumerate().take(ch_len) {
                outputs[ch][i] = S::from_f64(inputs[ch][i].to_f64() * gain.to_f64());
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
