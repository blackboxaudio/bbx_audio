//! Overdrive distortion effect block.

use bbx_core::flush_denormal_f64;

use crate::{
    block::{Block, DEFAULT_EFFECTOR_INPUT_COUNT, DEFAULT_EFFECTOR_OUTPUT_COUNT},
    context::DspContext,
    graph::MAX_BLOCK_OUTPUTS,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    smoothing::LinearSmoothedValue,
};

/// Maximum buffer size for stack-allocated smoothing cache.
const MAX_BUFFER_SIZE: usize = 4096;

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
    filter_state: [f64; MAX_BLOCK_OUTPUTS],
    filter_coefficient: f64,

    /// Smoothed drive value for click-free changes.
    drive_smoother: LinearSmoothedValue<S>,
    /// Smoothed level value for click-free changes.
    level_smoother: LinearSmoothedValue<S>,
}

impl<S: Sample> OverdriveBlock<S> {
    /// Create an `OverdriveBlock` with a given drive multiplier, level, tone (brightness), and sample rate.
    pub fn new(drive: f64, level: f64, tone: f64, sample_rate: f64) -> Self {
        let level_val = level.clamp(0.0, 1.0);

        let mut overdrive = Self {
            drive: Parameter::Constant(S::from_f64(drive)),
            level: Parameter::Constant(S::from_f64(level)),
            tone,
            filter_state: [0.0; MAX_BLOCK_OUTPUTS],
            filter_coefficient: 0.0,
            drive_smoother: LinearSmoothedValue::new(S::from_f64(drive)),
            level_smoother: LinearSmoothedValue::new(S::from_f64(level_val)),
        };
        overdrive.update_filter(sample_rate);
        overdrive
    }

    fn update_filter(&mut self, sample_rate: f64) {
        // Tone control: 0.0 = darker (300Hz), 1.0 = brighter (3KHz)
        let cutoff = 300.0 + (self.tone + 2700.0);
        self.filter_coefficient = 1.0 - (-2.0 * S::PI.to_f64() * cutoff / sample_rate).exp();
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
        // The 1.5 factor adjusts the "knee" of the saturation curve
        (x * 1.5).tanh() / 1.5
    }
}

impl<S: Sample> Block<S> for OverdriveBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let target_drive = S::from_f64(self.drive.get_value(modulation_values).to_f64());
        let target_level = S::from_f64(self.level.get_value(modulation_values).to_f64().clamp(0.0, 1.0));

        if (target_drive - self.drive_smoother.target()).abs() > S::EPSILON {
            self.drive_smoother.set_target_value(target_drive);
        }
        if (target_level - self.level_smoother.target()).abs() > S::EPSILON {
            self.level_smoother.set_target_value(target_level);
        }

        let len = inputs.first().map_or(0, |ch| ch.len().min(context.buffer_size));
        debug_assert!(len <= MAX_BUFFER_SIZE, "buffer_size exceeds MAX_BUFFER_SIZE");

        let mut drive_values: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];
        let mut level_values: [S; MAX_BUFFER_SIZE] = [S::ZERO; MAX_BUFFER_SIZE];

        for i in 0..len {
            drive_values[i] = self.drive_smoother.get_next_value();
            level_values[i] = self.level_smoother.get_next_value();
        }

        for (ch, input_buffer) in inputs.iter().enumerate() {
            if ch >= outputs.len() || ch >= MAX_BLOCK_OUTPUTS {
                break;
            }
            let ch_len = input_buffer.len().min(len);
            for (sample_index, sample_value) in input_buffer.iter().enumerate().take(ch_len) {
                let drive = drive_values[sample_index];
                let level = level_values[sample_index];

                let driven = sample_value.to_f64() * drive.to_f64();
                let clipped = self.asymmetric_saturation(driven);

                self.filter_state[ch] += self.filter_coefficient * (clipped - self.filter_state[ch]);
                self.filter_state[ch] = flush_denormal_f64(self.filter_state[ch]);
                outputs[ch][sample_index] = S::from_f64(self.filter_state[ch] * level.to_f64());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 6,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Surround51,
        }
    }

    #[test]
    fn test_overdrive_6_channels() {
        let mut overdrive = OverdriveBlock::<f32>::new(2.0, 0.8, 0.5, 44100.0);
        let context = test_context(4);

        let input: [[f32; 4]; 6] = [[0.5; 4]; 6];
        let mut outputs: [[f32; 4]; 6] = [[0.0; 4]; 6];

        let input_refs: Vec<&[f32]> = input.iter().map(|ch| ch.as_slice()).collect();
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut().map(|ch| ch.as_mut_slice()).collect();

        overdrive.process(&input_refs, &mut output_refs, &[], &context);

        for ch in 0..6 {
            assert!(outputs[ch][3].abs() > 0.0, "Channel {ch} should have output");
        }
    }

    #[test]
    fn test_overdrive_independent_channel_state() {
        let mut overdrive = OverdriveBlock::<f32>::new(3.0, 1.0, 0.5, 44100.0);
        let context = test_context(64);

        let mut input: [[f32; 64]; 4] = [[0.0; 64]; 4];
        input[0] = [0.8; 64];
        input[1] = [0.0; 64];
        input[2] = [0.4; 64];
        input[3] = [-0.4; 64];

        let mut outputs: [[f32; 64]; 4] = [[0.0; 64]; 4];

        let input_refs: Vec<&[f32]> = input.iter().map(|ch| ch.as_slice()).collect();
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut().map(|ch| ch.as_mut_slice()).collect();

        overdrive.process(&input_refs, &mut output_refs, &[], &context);

        assert!(outputs[0][63].abs() > outputs[1][63].abs());
        assert!(outputs[2][63].abs() < outputs[0][63].abs());
        assert!(outputs[3][63] < 0.0);
    }

    #[test]
    fn test_overdrive_input_output_counts_f32() {
        let overdrive = OverdriveBlock::<f32>::new(2.0, 0.8, 0.5, 44100.0);
        assert_eq!(overdrive.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(overdrive.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_overdrive_input_output_counts_f64() {
        let overdrive = OverdriveBlock::<f64>::new(2.0, 0.8, 0.5, 44100.0);
        assert_eq!(overdrive.input_count(), DEFAULT_EFFECTOR_INPUT_COUNT);
        assert_eq!(overdrive.output_count(), DEFAULT_EFFECTOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_overdrive_basic_f64() {
        let mut overdrive = OverdriveBlock::<f64>::new(2.0, 0.8, 0.5, 44100.0);
        let context = test_context(64);

        let input: [f64; 64] = [0.5; 64];
        let mut output: [f64; 64] = [0.0; 64];

        let inputs: [&[f64]; 1] = [&input];
        let mut outputs: [&mut [f64]; 1] = [&mut output];

        overdrive.process(&inputs, &mut outputs, &[], &context);

        assert!(output[63].abs() > 0.0, "Overdrive should produce output");
        assert!(output[63] <= 1.0, "Overdrive output should be bounded");
    }

    #[test]
    fn test_overdrive_modulation_outputs_empty() {
        let overdrive = OverdriveBlock::<f32>::new(2.0, 0.8, 0.5, 44100.0);
        assert!(overdrive.modulation_outputs().is_empty());
    }

    #[test]
    fn test_overdrive_asymmetric_saturation() {
        let mut overdrive = OverdriveBlock::<f32>::new(5.0, 1.0, 0.5, 44100.0);
        let context = test_context(64);

        let pos_input: [f32; 64] = [0.8; 64];
        let neg_input: [f32; 64] = [-0.8; 64];
        let mut pos_output: [f32; 64] = [0.0; 64];
        let mut neg_output: [f32; 64] = [0.0; 64];

        let pos_inputs: [&[f32]; 1] = [&pos_input];
        let mut pos_outputs: [&mut [f32]; 1] = [&mut pos_output];
        overdrive.process(&pos_inputs, &mut pos_outputs, &[], &context);

        let mut overdrive2 = OverdriveBlock::<f32>::new(5.0, 1.0, 0.5, 44100.0);
        let neg_inputs: [&[f32]; 1] = [&neg_input];
        let mut neg_outputs: [&mut [f32]; 1] = [&mut neg_output];
        overdrive2.process(&neg_inputs, &mut neg_outputs, &[], &context);

        assert!(
            pos_output[63].abs() != neg_output[63].abs(),
            "Asymmetric saturation should produce different magnitudes for +/- inputs"
        );
    }
}
