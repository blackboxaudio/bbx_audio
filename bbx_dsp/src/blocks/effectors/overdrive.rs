//! Overdrive distortion effect block.

use std::marker::PhantomData;

use bbx_core::flush_denormal_f64;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// An overdrive distortion effect with asymmetric soft clipping.
///
/// Uses hyperbolic tangent saturation with different curves for positive
/// and negative signal halves, creating a warm, tube-like distortion character.
/// Includes a one-pole lowpass filter for tone control.
pub struct OverdriveBlock<S: Sample> {
    /// Drive amount (gain before clipping, typically 1.0-10.0).
    pub drive: Parameter<S>,

    /// Output level (0.0-1.0).
    pub level: Parameter<S>,

    tone: f64,
    filter_state: f64,
    filter_coefficient: f64,

    /// Smoothed drive value for click-free changes.
    drive_smoother: LinearSmoothedValue,
    /// Smoothed level value for click-free changes.
    level_smoother: LinearSmoothedValue,

    _phantom_data: PhantomData<S>,
}

impl<S: Sample> OverdriveBlock<S> {
    /// Create an `OverdriveBlock` with a given drive multiplier, level, tone (brightness), and sample rate.
    pub fn new(drive: S, level: S, tone: f64, sample_rate: f64) -> Self {
        let drive_val = drive.to_f64();
        let level_val = level.to_f64().clamp(0.0, 1.0);

        let mut overdrive = Self {
            drive: Parameter::Constant(drive),
            level: Parameter::Constant(level),
            tone,
            filter_state: 0.0,
            filter_coefficient: 0.0,
            drive_smoother: LinearSmoothedValue::new(drive_val as f32),
            level_smoother: LinearSmoothedValue::new(level_val as f32),
            _phantom_data: PhantomData,
        };
        overdrive.update_filter(sample_rate);
        overdrive
    }

    fn update_filter(&mut self, sample_rate: f64) {
        // Tone control: 0.0 = darker (300Hz), 1.0 = brighter (3KHz)
        let cutoff = 300.0 + (self.tone + 2700.0);
        self.filter_coefficient = 1.0 - (-2.0 * std::f64::consts::PI * cutoff / sample_rate).exp();
    }

    #[inline]
    fn asymmetric_saturation(&self, x: f64) -> f64 {
        if x > 0.0 {
            // Positive half: softer clipping (more headroom)
            self.soft_clip(x * 0.7) * 1.4
        } else {
            // Negative half: harder clipping (more compression)
            self.soft_clip(x * 1.2) * 0.8
        }
    }

    #[inline]
    fn soft_clip(&self, x: f64) -> f64 {
        // Hyperbolic tangent function provides smooth saturation.
        // The 1.5 factor adjusts the "knee" of the saturation curve.
        (x * 1.5).tanh() / 1.5
    }
}

impl<S: Sample> Block<S> for OverdriveBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        // Get target values and set up smoothing
        let target_drive = self.drive.get_value(modulation_values).to_f64() as f32;
        let target_level = self.level.get_value(modulation_values).to_f64().clamp(0.0, 1.0) as f32;

        if (target_drive - self.drive_smoother.target()).abs() > 1e-6 {
            self.drive_smoother.set_target_value(target_drive);
        }
        if (target_level - self.level_smoother.target()).abs() > 1e-6 {
            self.level_smoother.set_target_value(target_level);
        }

        for (input_index, input_buffer) in inputs.iter().enumerate() {
            // Clone smoothers for each channel to get same curve
            let mut drive_sm = self.drive_smoother.clone();
            let mut level_sm = self.level_smoother.clone();

            for (sample_index, sample_value) in input_buffer.iter().enumerate() {
                let drive = drive_sm.get_next_value() as f64;
                let level = level_sm.get_next_value() as f64;

                let driven = sample_value.to_f64() * drive;
                let clipped = self.asymmetric_saturation(driven);

                self.filter_state += self.filter_coefficient * (clipped - self.filter_state);
                // Flush denormals to prevent CPU slowdown during quiet passages
                self.filter_state = flush_denormal_f64(self.filter_state);
                outputs[input_index][sample_index] = S::from_f64(self.filter_state * level);
            }
        }

        // Advance main smoothers
        self.drive_smoother.skip(context.buffer_size as i32);
        self.level_smoother.skip(context.buffer_size as i32);
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
