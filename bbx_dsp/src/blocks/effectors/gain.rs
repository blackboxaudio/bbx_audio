//! Gain control block with dB input.

use std::marker::PhantomData;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// A gain control block that applies amplitude scaling.
///
/// Level is specified in decibels (dB).
pub struct GainBlock<S: Sample> {
    /// Gain level in dB (-60 to +30).
    pub level_db: Parameter<S>,

    _phantom: PhantomData<S>,
}

impl<S: Sample> GainBlock<S> {
    /// Create a new `GainBlock` with the given level in dB.
    pub fn new(level_db: S) -> Self {
        Self {
            level_db: Parameter::Constant(level_db),
            _phantom: PhantomData,
        }
    }

    /// Create a unity gain (0 dB) block.
    pub fn unity() -> Self {
        Self::new(S::ZERO)
    }

    /// Convert dB to linear gain.
    #[inline]
    fn db_to_linear(db: f64) -> f64 {
        10.0_f64.powf(db / 20.0)
    }
}

impl<S: Sample> Block<S> for GainBlock<S> {
    fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        _context: &DspContext,
    ) {
        let level_db = self.level_db.get_value(modulation_values).to_f64();
        let gain = Self::db_to_linear(level_db);

        for (ch, input) in inputs.iter().enumerate() {
            if ch >= outputs.len() {
                break;
            }

            for (i, &sample) in input.iter().enumerate() {
                if i >= outputs[ch].len() {
                    break;
                }
                outputs[ch][i] = S::from_f64(sample.to_f64() * gain);
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
