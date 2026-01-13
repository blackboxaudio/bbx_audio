//! ADSR envelope generator block.

use crate::{
    block::{Block, DEFAULT_MODULATOR_INPUT_COUNT, DEFAULT_MODULATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// ADSR envelope stages.
///
/// The envelope progresses through: Idle -> Attack -> Decay -> Sustain -> Release -> Idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR envelope generator block for amplitude and parameter modulation.
pub struct EnvelopeBlock<S: Sample> {
    /// Attack time in seconds.
    pub attack: Parameter<S>,
    /// Decay time in seconds.
    pub decay: Parameter<S>,
    /// Sustain level (0.0 - 1.0).
    pub sustain: Parameter<S>,
    /// Release time in seconds.
    pub release: Parameter<S>,

    stage: EnvelopeStage,
    level: f64,
    stage_time: f64,
    release_level: f64,
}

impl<S: Sample> EnvelopeBlock<S> {
    const MODULATION_OUTPUTS: &'static [ModulationOutput] = &[ModulationOutput {
        name: "Envelope",
        min_value: 0.0,
        max_value: 1.0,
    }];

    /// Minimum envelope time in seconds.
    const MIN_TIME: f64 = 0.001;
    /// Maximum envelope time in seconds.
    const MAX_TIME: f64 = 10.0;
    /// Envelope floor threshold (~-120dB) for reliable release termination.
    const ENVELOPE_FLOOR: f64 = 1e-6;

    /// Create an `EnvelopeBlock` with given ADSR parameters.
    /// Times are in seconds, sustain is a level from 0.0 to 1.0.
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self {
        Self {
            attack: Parameter::Constant(S::from_f64(attack)),
            decay: Parameter::Constant(S::from_f64(decay)),
            sustain: Parameter::Constant(S::from_f64(sustain)),
            release: Parameter::Constant(S::from_f64(release)),
            stage: EnvelopeStage::Idle,
            level: 0.0,
            stage_time: 0.0,
            release_level: 0.0,
        }
    }

    /// Trigger the envelope (note on).
    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.stage_time = 0.0;
    }

    /// Release the envelope (note off).
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.release_level = self.level;
            self.stage = EnvelopeStage::Release;
            self.stage_time = 0.0;
        }
    }

    /// Reset the envelope to idle state.
    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.level = 0.0;
        self.stage_time = 0.0;
        self.release_level = 0.0;
    }

    #[inline]
    fn clamp_time(time: f64) -> f64 {
        time.clamp(Self::MIN_TIME, Self::MAX_TIME)
    }
}

impl<S: Sample> Block<S> for EnvelopeBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let attack_time = Self::clamp_time(self.attack.get_value(modulation_values).to_f64());
        let decay_time = Self::clamp_time(self.decay.get_value(modulation_values).to_f64());
        let sustain_level = self.sustain.get_value(modulation_values).to_f64().clamp(0.0, 1.0);
        let release_time = Self::clamp_time(self.release.get_value(modulation_values).to_f64());

        let time_per_sample = 1.0 / context.sample_rate;

        for sample_index in 0..context.buffer_size {
            match self.stage {
                EnvelopeStage::Idle => {
                    self.level = 0.0;
                }
                EnvelopeStage::Attack => {
                    self.level = self.stage_time / attack_time;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.stage = EnvelopeStage::Decay;
                        self.stage_time = 0.0;
                    }
                }
                EnvelopeStage::Decay => {
                    let decay_progress = self.stage_time / decay_time;
                    self.level = 1.0 - (1.0 - sustain_level) * decay_progress;
                    if self.level <= sustain_level {
                        self.level = sustain_level;
                        self.stage = EnvelopeStage::Sustain;
                        self.stage_time = 0.0;
                    }
                }
                EnvelopeStage::Sustain => {
                    self.level = sustain_level;
                }
                EnvelopeStage::Release => {
                    let release_progress = self.stage_time / release_time;
                    self.level = self.release_level * (1.0 - release_progress);
                    // Avoids floating-point precision issues with exact zero comparison
                    if self.level <= Self::ENVELOPE_FLOOR {
                        self.level = 0.0;
                        self.stage = EnvelopeStage::Idle;
                        self.stage_time = 0.0;
                    }
                }
            }

            outputs[0][sample_index] = S::from_f64(self.level);

            if self.stage != EnvelopeStage::Idle && self.stage != EnvelopeStage::Sustain {
                self.stage_time += time_per_sample;
            }
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_MODULATOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_MODULATOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        Self::MODULATION_OUTPUTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    fn test_context(buffer_size: usize, sample_rate: f64) -> DspContext {
        DspContext {
            sample_rate,
            num_channels: 1,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Mono,
        }
    }

    fn process_envelope<S: Sample>(env: &mut EnvelopeBlock<S>, context: &DspContext) -> Vec<S> {
        let inputs: [&[S]; 0] = [];
        let mut output = vec![S::ZERO; context.buffer_size];
        let mut outputs: [&mut [S]; 1] = [&mut output];
        env.process(&inputs, &mut outputs, &[], context);
        output
    }

    #[test]
    fn test_envelope_input_output_counts_f32() {
        let env = EnvelopeBlock::<f32>::new(0.01, 0.1, 0.5, 0.2);
        assert_eq!(env.input_count(), DEFAULT_MODULATOR_INPUT_COUNT);
        assert_eq!(env.output_count(), DEFAULT_MODULATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_envelope_input_output_counts_f64() {
        let env = EnvelopeBlock::<f64>::new(0.01, 0.1, 0.5, 0.2);
        assert_eq!(env.input_count(), DEFAULT_MODULATOR_INPUT_COUNT);
        assert_eq!(env.output_count(), DEFAULT_MODULATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_envelope_modulation_output_f32() {
        let env = EnvelopeBlock::<f32>::new(0.01, 0.1, 0.5, 0.2);
        let outputs = env.modulation_outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].name, "Envelope");
        assert!((outputs[0].min_value - 0.0).abs() < 1e-10);
        assert!((outputs[0].max_value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_envelope_idle_produces_zero_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.01, 0.1, 0.5, 0.2);
        let context = test_context(512, 44100.0);
        let output = process_envelope(&mut env, &context);

        for (i, &sample) in output.iter().enumerate() {
            assert!(
                sample.abs() < 1e-6,
                "Idle envelope should produce zero: output[{}]={}",
                i,
                sample
            );
        }
    }

    #[test]
    fn test_envelope_idle_produces_zero_f64() {
        let mut env = EnvelopeBlock::<f64>::new(0.01, 0.1, 0.5, 0.2);
        let context = test_context(512, 44100.0);
        let output = process_envelope(&mut env, &context);

        for (i, &sample) in output.iter().enumerate() {
            assert!(
                sample.abs() < 1e-12,
                "Idle envelope should produce zero: output[{}]={}",
                i,
                sample
            );
        }
    }

    #[test]
    fn test_envelope_output_range_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.01, 0.05, 0.7, 0.1);
        let context = test_context(512, 44100.0);

        env.note_on();

        for _ in 0..100 {
            let output = process_envelope(&mut env, &context);
            for &sample in &output {
                assert!(
                    sample >= 0.0 && sample <= 1.0,
                    "Envelope should be in [0, 1] range: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_envelope_output_range_f64() {
        let mut env = EnvelopeBlock::<f64>::new(0.01, 0.05, 0.7, 0.1);
        let context = test_context(512, 44100.0);

        env.note_on();

        for _ in 0..100 {
            let output = process_envelope(&mut env, &context);
            for &sample in &output {
                assert!(
                    sample >= 0.0 && sample <= 1.0,
                    "Envelope should be in [0, 1] range: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_envelope_attack_rises_f32() {
        let attack_time = 0.1;
        let mut env = EnvelopeBlock::<f32>::new(attack_time, 0.1, 0.5, 0.1);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        let output = process_envelope(&mut env, &context);

        assert!(output[0] < output[output.len() - 1], "Attack should rise over time");
        assert!(output[0] < 0.5, "Attack should start low");
    }

    #[test]
    fn test_envelope_attack_rises_f64() {
        let attack_time = 0.1;
        let mut env = EnvelopeBlock::<f64>::new(attack_time, 0.1, 0.5, 0.1);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        let output = process_envelope(&mut env, &context);

        assert!(output[0] < output[output.len() - 1], "Attack should rise over time");
        assert!(output[0] < 0.5, "Attack should start low");
    }

    #[test]
    fn test_envelope_reaches_peak_f32() {
        let attack_time = 0.01;
        let mut env = EnvelopeBlock::<f32>::new(attack_time, 0.5, 0.5, 0.5);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..10 {
            let output = process_envelope(&mut env, &context);
            let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x));
            if (max - 1.0).abs() < 0.05 {
                return;
            }
        }
        panic!("Envelope should reach peak of 1.0 during attack");
    }

    #[test]
    fn test_envelope_reaches_peak_f64() {
        let attack_time = 0.01;
        let mut env = EnvelopeBlock::<f64>::new(attack_time, 0.5, 0.5, 0.5);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..10 {
            let output = process_envelope(&mut env, &context);
            let max = output.iter().fold(0.0f64, |acc, &x| acc.max(x));
            if (max - 1.0).abs() < 0.05 {
                return;
            }
        }
        panic!("Envelope should reach peak of 1.0 during attack");
    }

    #[test]
    fn test_envelope_sustain_level_f32() {
        let sustain = 0.6;
        let mut env = EnvelopeBlock::<f32>::new(0.001, 0.001, sustain, 0.5);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..20 {
            let _ = process_envelope(&mut env, &context);
        }

        let output = process_envelope(&mut env, &context);
        let avg: f32 = output.iter().sum::<f32>() / output.len() as f32;

        assert!(
            (avg - sustain as f32).abs() < 0.05,
            "Sustain should hold at sustain level: expected={}, got={}",
            sustain,
            avg
        );
    }

    #[test]
    fn test_envelope_sustain_level_f64() {
        let sustain = 0.6;
        let mut env = EnvelopeBlock::<f64>::new(0.001, 0.001, sustain, 0.5);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..20 {
            let _ = process_envelope(&mut env, &context);
        }

        let output = process_envelope(&mut env, &context);
        let avg: f64 = output.iter().sum::<f64>() / output.len() as f64;

        assert!(
            (avg - sustain).abs() < 0.05,
            "Sustain should hold at sustain level: expected={}, got={}",
            sustain,
            avg
        );
    }

    #[test]
    fn test_envelope_release_falls_f32() {
        let sustain = 0.7;
        let mut env = EnvelopeBlock::<f32>::new(0.001, 0.001, sustain, 0.1);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..10 {
            let _ = process_envelope(&mut env, &context);
        }

        env.note_off();

        let output1 = process_envelope(&mut env, &context);
        let output2 = process_envelope(&mut env, &context);

        let first_avg: f32 = output1.iter().sum::<f32>() / output1.len() as f32;
        let second_avg: f32 = output2.iter().sum::<f32>() / output2.len() as f32;

        assert!(
            first_avg > second_avg,
            "Release should fall over time: first={}, second={}",
            first_avg,
            second_avg
        );
    }

    #[test]
    fn test_envelope_release_falls_f64() {
        let sustain = 0.7_f64;
        let mut env = EnvelopeBlock::<f64>::new(0.001, 0.001, sustain, 0.1);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..10 {
            let _ = process_envelope(&mut env, &context);
        }

        env.note_off();

        let output1 = process_envelope(&mut env, &context);
        let output2 = process_envelope(&mut env, &context);

        let first_avg: f64 = output1.iter().sum::<f64>() / output1.len() as f64;
        let second_avg: f64 = output2.iter().sum::<f64>() / output2.len() as f64;

        assert!(
            first_avg > second_avg,
            "Release should fall over time: first={}, second={}",
            first_avg,
            second_avg
        );
    }

    #[test]
    fn test_envelope_release_returns_to_zero_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.001, 0.001, 0.5, 0.01);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..5 {
            let _ = process_envelope(&mut env, &context);
        }

        env.note_off();

        for _ in 0..20 {
            let output = process_envelope(&mut env, &context);
            let last = output[output.len() - 1];
            if last.abs() < 1e-5 {
                return;
            }
        }
        panic!("Release should return to zero");
    }

    #[test]
    fn test_envelope_release_returns_to_zero_f64() {
        let mut env = EnvelopeBlock::<f64>::new(0.001, 0.001, 0.5, 0.01);
        let sample_rate = 44100.0;
        let context = test_context(512, sample_rate);

        env.note_on();

        for _ in 0..5 {
            let _ = process_envelope(&mut env, &context);
        }

        env.note_off();

        for _ in 0..20 {
            let output = process_envelope(&mut env, &context);
            let last = output[output.len() - 1];
            if last.abs() < 1e-5 {
                return;
            }
        }
        panic!("Release should return to zero");
    }

    #[test]
    fn test_envelope_reset_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.1, 0.1, 0.5, 0.1);
        let context = test_context(512, 44100.0);

        env.note_on();
        let _ = process_envelope(&mut env, &context);

        env.reset();

        let output = process_envelope(&mut env, &context);
        for &sample in &output {
            assert!(sample.abs() < 1e-6, "Reset should return to zero");
        }
    }

    #[test]
    fn test_envelope_reset_f64() {
        let mut env = EnvelopeBlock::<f64>::new(0.1, 0.1, 0.5, 0.1);
        let context = test_context(512, 44100.0);

        env.note_on();
        let _ = process_envelope(&mut env, &context);

        env.reset();

        let output = process_envelope(&mut env, &context);
        for &sample in &output {
            assert!(sample.abs() < 1e-12, "Reset should return to zero");
        }
    }

    #[test]
    fn test_envelope_note_off_from_idle_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.1, 0.1, 0.5, 0.1);
        let context = test_context(512, 44100.0);

        env.note_off();

        let output = process_envelope(&mut env, &context);
        for &sample in &output {
            assert!(sample.abs() < 1e-6, "note_off from idle should stay at zero");
        }
    }

    #[test]
    fn test_envelope_retrigger_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.001, 0.001, 0.5, 0.1);
        let context = test_context(512, 44100.0);

        env.note_on();
        for _ in 0..10 {
            let _ = process_envelope(&mut env, &context);
        }

        env.note_on();

        let output = process_envelope(&mut env, &context);
        assert!(
            output.iter().any(|&x| x < 0.5),
            "Retrigger should restart attack from beginning"
        );
    }

    #[test]
    fn test_envelope_time_clamping_f32() {
        let mut env = EnvelopeBlock::<f32>::new(0.0001, 0.0001, 0.5, 0.0001);
        let context = test_context(512, 44100.0);

        env.note_on();

        for _ in 0..100 {
            let output = process_envelope(&mut env, &context);
            for &sample in &output {
                assert!(
                    sample >= 0.0 && sample <= 1.0,
                    "Very short times should still work: {}",
                    sample
                );
            }
        }
    }
}
