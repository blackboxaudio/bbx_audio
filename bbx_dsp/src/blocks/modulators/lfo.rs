//! Low-frequency oscillator (LFO) block for parameter modulation.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd;
use crate::{
    block::{Block, DEFAULT_MODULATOR_INPUT_COUNT, DEFAULT_MODULATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar},
};

/// A low-frequency oscillator for modulating block parameters.
///
/// Generates control signals (typically < 20 Hz) using standard waveforms.
/// Output range is -depth to +depth, centered at zero.
pub struct LfoBlock<S: Sample> {
    /// LFO frequency in Hz (typically 0.01-20 Hz).
    pub frequency: Parameter<S>,

    /// Modulation depth (output amplitude).
    pub depth: Parameter<S>,

    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> LfoBlock<S> {
    const MODULATION_OUTPUTS: &'static [ModulationOutput] = &[ModulationOutput {
        name: "LFO",
        min_value: -1.0,
        max_value: 1.0,
    }];

    /// Create an `LfoBlock` with a given frequency, depth, waveform, and optional seed (used for noise waveforms).
    pub fn new(frequency: f64, depth: f64, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            frequency: Parameter::Constant(S::from_f64(frequency)),
            depth: Parameter::Constant(S::from_f64(depth)),
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
        }
    }
}

impl<S: Sample> Block<S> for LfoBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let frequency = self.frequency.get_value(modulation_values);
        let depth = self.depth.get_value(modulation_values).to_f64();
        let phase_increment = frequency.to_f64() / context.sample_rate * S::TAU.to_f64();

        #[cfg(feature = "simd")]
        {
            use crate::waveform::DEFAULT_DUTY_CYCLE;

            if !matches!(self.waveform, Waveform::Noise) {
                let buffer_size = context.buffer_size;
                let chunks = buffer_size / SIMD_LANES;
                let remainder_start = chunks * SIMD_LANES;
                let chunk_phase_step = phase_increment * SIMD_LANES as f64;
                let depth_s = S::from_f64(depth);
                let depth_vec = S::simd_splat(depth_s);

                let base_phase = S::simd_splat(S::from_f64(self.phase));
                let sample_inc_simd = S::simd_splat(S::from_f64(phase_increment));
                let mut phases = base_phase + S::simd_lane_offsets() * sample_inc_simd;
                let chunk_inc_simd = S::simd_splat(S::from_f64(chunk_phase_step));
                let duty = S::from_f64(DEFAULT_DUTY_CYCLE);
                let two_pi = S::simd_splat(S::TAU);
                let inv_two_pi = S::simd_splat(S::INV_TAU);
                let phase_inc_normalized = S::from_f64(phase_increment * S::INV_TAU.to_f64());
                let tau = S::TAU.to_f64();
                let inv_tau = 1.0 / tau;

                for chunk_idx in 0..chunks {
                    let phases_array = S::simd_to_array(phases);
                    let phases_normalized: [S; SIMD_LANES] = [
                        S::from_f64(phases_array[0].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[1].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[2].to_f64().rem_euclid(tau) * inv_tau),
                        S::from_f64(phases_array[3].to_f64().rem_euclid(tau) * inv_tau),
                    ];

                    if let Some(samples) = generate_waveform_samples_simd::<S>(
                        self.waveform,
                        phases,
                        phases_normalized,
                        phase_inc_normalized,
                        duty,
                        two_pi,
                        inv_two_pi,
                    ) {
                        let samples_vec = S::simd_from_slice(&samples);
                        let scaled = samples_vec * depth_vec;
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&S::simd_to_array(scaled));
                    }

                    phases = phases + chunk_inc_simd;
                }

                self.phase += chunk_phase_step * chunks as f64;
                self.phase = self.phase.rem_euclid(S::TAU.to_f64());

                process_waveform_scalar(
                    &mut outputs[0][remainder_start..],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    depth,
                );
            } else {
                process_waveform_scalar(
                    outputs[0],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    depth,
                );
            }
        }

        #[cfg(not(feature = "simd"))]
        {
            process_waveform_scalar(
                outputs[0],
                self.waveform,
                &mut self.phase,
                phase_increment,
                &mut self.rng,
                depth,
            );
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

    fn process_lfo<S: Sample>(lfo: &mut LfoBlock<S>, context: &DspContext) -> Vec<S> {
        let inputs: [&[S]; 0] = [];
        let mut output = vec![S::ZERO; context.buffer_size];
        let mut outputs: [&mut [S]; 1] = [&mut output];
        lfo.process(&inputs, &mut outputs, &[], context);
        output
    }

    #[test]
    fn test_lfo_input_output_counts_f32() {
        let lfo = LfoBlock::<f32>::new(1.0, 1.0, Waveform::Sine, None);
        assert_eq!(lfo.input_count(), DEFAULT_MODULATOR_INPUT_COUNT);
        assert_eq!(lfo.output_count(), DEFAULT_MODULATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_lfo_input_output_counts_f64() {
        let lfo = LfoBlock::<f64>::new(1.0, 1.0, Waveform::Sine, None);
        assert_eq!(lfo.input_count(), DEFAULT_MODULATOR_INPUT_COUNT);
        assert_eq!(lfo.output_count(), DEFAULT_MODULATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_lfo_modulation_output_f32() {
        let lfo = LfoBlock::<f32>::new(1.0, 1.0, Waveform::Sine, None);
        let outputs = lfo.modulation_outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].name, "LFO");
        assert!((outputs[0].min_value - (-1.0)).abs() < 1e-10);
        assert!((outputs[0].max_value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lfo_output_range_unity_depth_f32() {
        let mut lfo = LfoBlock::<f32>::new(1.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "LFO with depth=1 should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_output_range_unity_depth_f64() {
        let mut lfo = LfoBlock::<f64>::new(1.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "LFO with depth=1 should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_depth_scaling_f32() {
        let depth = 0.5;
        let mut lfo = LfoBlock::<f32>::new(1.0, depth, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -depth as f32 * 1.1 && sample <= depth as f32 * 1.1,
                    "LFO with depth={} should be in [{}, {}]: {}",
                    depth,
                    -depth,
                    depth,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_depth_scaling_f64() {
        let depth = 0.5;
        let mut lfo = LfoBlock::<f64>::new(1.0, depth, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -depth * 1.1 && sample <= depth * 1.1,
                    "LFO with depth={} should be in [{}, {}]: {}",
                    depth,
                    -depth,
                    depth,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_sine_produces_variation_f32() {
        let mut lfo = LfoBlock::<f32>::new(5.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        let output = process_lfo(&mut lfo, &context);

        let min = output.iter().fold(f32::MAX, |acc, &x| acc.min(x));
        let max = output.iter().fold(f32::MIN, |acc, &x| acc.max(x));

        assert!(
            max - min > 0.1,
            "LFO should produce variation: min={}, max={}",
            min,
            max
        );
    }

    #[test]
    fn test_lfo_sine_produces_variation_f64() {
        let mut lfo = LfoBlock::<f64>::new(5.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        let output = process_lfo(&mut lfo, &context);

        let min = output.iter().fold(f64::MAX, |acc, &x| acc.min(x));
        let max = output.iter().fold(f64::MIN, |acc, &x| acc.max(x));

        assert!(
            max - min > 0.1,
            "LFO should produce variation: min={}, max={}",
            min,
            max
        );
    }

    #[test]
    fn test_lfo_low_frequency_f32() {
        let mut lfo = LfoBlock::<f32>::new(0.1, 1.0, Waveform::Sine, Some(42));
        let context = test_context(4096, 44100.0);

        let mut all_samples = Vec::new();
        for _ in 0..50 {
            let output = process_lfo(&mut lfo, &context);
            all_samples.extend(output);
        }

        let has_positive = all_samples.iter().any(|&x| x > 0.1);
        let has_negative = all_samples.iter().any(|&x| x < -0.1);

        assert!(
            has_positive || has_negative,
            "Very low frequency LFO should still produce signal"
        );
    }

    #[test]
    fn test_lfo_high_frequency_f32() {
        let mut lfo = LfoBlock::<f32>::new(20.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        let output = process_lfo(&mut lfo, &context);

        let min = output.iter().fold(f32::MAX, |acc, &x| acc.min(x));
        let max = output.iter().fold(f32::MIN, |acc, &x| acc.max(x));

        assert!(max - min > 0.5, "High frequency LFO should oscillate within buffer");
    }

    #[test]
    fn test_lfo_square_output_range_f32() {
        let mut lfo = LfoBlock::<f32>::new(2.0, 1.0, Waveform::Square, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Square LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_square_output_range_f64() {
        let mut lfo = LfoBlock::<f64>::new(2.0, 1.0, Waveform::Square, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Square LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_sawtooth_output_range_f32() {
        let mut lfo = LfoBlock::<f32>::new(2.0, 1.0, Waveform::Sawtooth, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Sawtooth LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_sawtooth_output_range_f64() {
        let mut lfo = LfoBlock::<f64>::new(2.0, 1.0, Waveform::Sawtooth, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Sawtooth LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_triangle_output_range_f32() {
        let mut lfo = LfoBlock::<f32>::new(2.0, 1.0, Waveform::Triangle, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Triangle LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_triangle_output_range_f64() {
        let mut lfo = LfoBlock::<f64>::new(2.0, 1.0, Waveform::Triangle, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.1 && sample <= 1.1,
                    "Triangle LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_noise_output_range_f32() {
        let mut lfo = LfoBlock::<f32>::new(1.0, 1.0, Waveform::Noise, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.0 && sample <= 1.0,
                    "Noise LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_noise_output_range_f64() {
        let mut lfo = LfoBlock::<f64>::new(1.0, 1.0, Waveform::Noise, Some(42));
        let context = test_context(512, 44100.0);

        for _ in 0..10 {
            let output = process_lfo(&mut lfo, &context);
            for &sample in &output {
                assert!(
                    sample >= -1.0 && sample <= 1.0,
                    "Noise LFO should be in [-1, 1]: {}",
                    sample
                );
            }
        }
    }

    #[test]
    fn test_lfo_deterministic_with_seed_f32() {
        let output1 = {
            let mut lfo = LfoBlock::<f32>::new(5.0, 1.0, Waveform::Sine, Some(42));
            let context = test_context(256, 44100.0);
            process_lfo(&mut lfo, &context)
        };

        let output2 = {
            let mut lfo = LfoBlock::<f32>::new(5.0, 1.0, Waveform::Sine, Some(42));
            let context = test_context(256, 44100.0);
            process_lfo(&mut lfo, &context)
        };

        for (a, b) in output1.iter().zip(output2.iter()) {
            assert!((a - b).abs() < 1e-6, "Same seed should produce identical output");
        }
    }

    #[test]
    fn test_lfo_deterministic_with_seed_f64() {
        let output1 = {
            let mut lfo = LfoBlock::<f64>::new(5.0, 1.0, Waveform::Sine, Some(42));
            let context = test_context(256, 44100.0);
            process_lfo(&mut lfo, &context)
        };

        let output2 = {
            let mut lfo = LfoBlock::<f64>::new(5.0, 1.0, Waveform::Sine, Some(42));
            let context = test_context(256, 44100.0);
            process_lfo(&mut lfo, &context)
        };

        for (a, b) in output1.iter().zip(output2.iter()) {
            assert!((a - b).abs() < 1e-12, "Same seed should produce identical output");
        }
    }

    #[test]
    fn test_lfo_zero_depth_f32() {
        let mut lfo = LfoBlock::<f32>::new(5.0, 0.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        let output = process_lfo(&mut lfo, &context);

        for &sample in &output {
            assert!(sample.abs() < 1e-6, "Zero depth LFO should produce zero: {}", sample);
        }
    }

    #[test]
    fn test_lfo_zero_depth_f64() {
        let mut lfo = LfoBlock::<f64>::new(5.0, 0.0, Waveform::Sine, Some(42));
        let context = test_context(512, 44100.0);

        let output = process_lfo(&mut lfo, &context);

        for &sample in &output {
            assert!(sample.abs() < 1e-12, "Zero depth LFO should produce zero: {}", sample);
        }
    }

    #[test]
    fn test_lfo_phase_continuity_f32() {
        let mut lfo = LfoBlock::<f32>::new(5.0, 1.0, Waveform::Sine, Some(42));
        let context = test_context(256, 44100.0);

        let output1 = process_lfo(&mut lfo, &context);
        let output2 = process_lfo(&mut lfo, &context);

        let last = output1[255];
        let first = output2[0];
        let diff = (last - first).abs();

        let samples_per_cycle = 44100.0 / 5.0;
        let expected_diff_per_sample = 2.0 / samples_per_cycle;

        assert!(
            diff < expected_diff_per_sample * 10.0,
            "Phase discontinuity detected: last={}, first={}, diff={}",
            last,
            first,
            diff
        );
    }
}
