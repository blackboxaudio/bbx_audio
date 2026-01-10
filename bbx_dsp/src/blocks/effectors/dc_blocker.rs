//! DC offset removal filter using a simple one-pole high-pass design.

use std::marker::PhantomData;

use bbx_core::flush_denormal_f64;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::ModulationOutput,
    sample::Sample,
};

/// A DC blocking filter that removes DC offset from audio signals.
///
/// Uses a first-order high-pass filter with approximately 5Hz cutoff.
pub struct DcBlockerBlock<S: Sample> {
    /// Whether the DC blocker is enabled.
    pub enabled: bool,

    // Filter state per channel (up to 2 channels)
    x_prev: [f64; 2],
    y_prev: [f64; 2],

    // Filter coefficient (~0.995 for 5Hz at 44.1kHz)
    coeff: f64,

    _phantom: PhantomData<S>,
}

impl<S: Sample> DcBlockerBlock<S> {
    /// Create a new `DcBlockerBlock`.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            x_prev: [0.0; 2],
            y_prev: [0.0; 2],
            coeff: 0.995, // Will be recalculated on prepare
            _phantom: PhantomData,
        }
    }

    /// Recalculate filter coefficient for the given sample rate.
    /// Targets approximately 5Hz cutoff frequency.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        // DC blocker coefficient: R = 1 - (2 * PI * fc / fs)
        // For fc = 5Hz, this gives approximately 0.9993 at 44.1kHz
        let cutoff_hz = 5.0;
        self.coeff = 1.0 - (2.0 * f64::PI * cutoff_hz / sample_rate);
        self.coeff = self.coeff.clamp(0.9, 0.9999);
    }

    /// Reset the filter state.
    pub fn reset(&mut self) {
        self.x_prev = [0.0; 2];
        self.y_prev = [0.0; 2];
    }
}

impl<S: Sample> Block<S> for DcBlockerBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        if !self.enabled {
            // Pass through unchanged
            for (ch, input) in inputs.iter().enumerate() {
                if ch < outputs.len() {
                    outputs[ch].copy_from_slice(input);
                }
            }
            return;
        }

        // Process each channel
        for (ch, input) in inputs.iter().enumerate() {
            if ch >= outputs.len() || ch >= 2 {
                break;
            }

            for (i, &sample) in input.iter().enumerate() {
                let x = sample.to_f64();

                // y[n] = x[n] - x[n-1] + R * y[n-1]
                let y = x - self.x_prev[ch] + self.coeff * self.y_prev[ch];

                self.x_prev[ch] = x;
                // Flush denormals to prevent CPU slowdown during quiet passages
                self.y_prev[ch] = flush_denormal_f64(y);

                outputs[ch][i] = S::from_f64(y);
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
