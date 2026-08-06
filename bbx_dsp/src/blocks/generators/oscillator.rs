//! Waveform oscillator block.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd;
use crate::{
    block::{Block, DEFAULT_GENERATOR_INPUT_COUNT, DEFAULT_GENERATOR_OUTPUT_COUNT},
    context::DspContext,
    math,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar},
};

/// A waveform oscillator for generating audio signals.
///
/// Supports standard waveforms (sine, square, sawtooth, triangle, pulse, noise).
/// Frequency can be controlled via parameter modulation or MIDI note messages.
///
/// Uses PolyBLEP/PolyBLAMP for band-limited output, reducing aliasing artifacts.
pub struct OscillatorBlock<S: Sample> {
    /// Base frequency in Hz (can be modulated).
    pub frequency: Parameter<S>,

    /// Pitch offset in semitones (for pitch bend/modulation).
    pub pitch_offset: Parameter<S>,

    base_frequency: S,
    midi_frequency: Option<S>,
    phase: f64,
    waveform: Waveform,
    rng: XorShiftRng,
}

impl<S: Sample> OscillatorBlock<S> {
    /// Create a new oscillator with the given frequency and waveform.
    pub fn new(frequency: f64, waveform: Waveform, seed: Option<u64>) -> Self {
        let freq = S::from_f64(frequency);
        Self {
            frequency: Parameter::Constant(freq),
            pitch_offset: Parameter::Constant(S::ZERO),
            base_frequency: freq,
            midi_frequency: None,
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
        }
    }

    /// Set the MIDI-controlled frequency (called by voice manager on note-on).
    pub fn set_midi_frequency(&mut self, frequency: S) {
        self.midi_frequency = Some(frequency);
    }

    /// Clear the MIDI frequency (called on note-off or when returning to parameter control).
    pub fn clear_midi_frequency(&mut self) {
        self.midi_frequency = None;
    }

    /// Set the waveform type.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let base = self.midi_frequency.unwrap_or(self.base_frequency);

        let freq_hz = match &self.frequency {
            Parameter::Constant(f) => self.midi_frequency.unwrap_or(*f),
            Parameter::Modulated(block_id) => {
                let mod_value = modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO);
                base + mod_value
            }
        };

        let pitch_offset_semitones = match &self.pitch_offset {
            Parameter::Constant(offset) => *offset,
            Parameter::Modulated(block_id) => modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO),
        };

        let freq = if pitch_offset_semitones != S::ZERO {
            let multiplier = S::from_f64(math::powf(2.0f64, pitch_offset_semitones.to_f64() / 12.0));
            freq_hz * multiplier
        } else {
            freq_hz
        };

        let phase_increment = freq.to_f64() / context.sample_rate * S::TAU.to_f64();

        #[cfg(feature = "simd")]
        {
            use crate::waveform::DEFAULT_DUTY_CYCLE;

            if !matches!(self.waveform, Waveform::Noise) {
                let buffer_size = context.buffer_size;
                let chunks = buffer_size / SIMD_LANES;
                let remainder_start = chunks * SIMD_LANES;
                let chunk_phase_step = phase_increment * SIMD_LANES as f64;

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
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&samples);
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
                    1.0,
                );
            } else {
                process_waveform_scalar(
                    outputs[0],
                    self.waveform,
                    &mut self.phase,
                    phase_increment,
                    &mut self.rng,
                    1.0,
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
                1.0,
            );
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        DEFAULT_GENERATOR_INPUT_COUNT
    }

    #[inline]
    fn output_count(&self) -> usize {
        DEFAULT_GENERATOR_OUTPUT_COUNT
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelLayout;

    // =============================================================================
    // Tolerance Constants
    // =============================================================================
    //
    // PolyBLEP anti-aliasing adds small overshoots near waveform discontinuities to
    // reduce aliasing artifacts. The tolerances below account for these overshoots:
    //
    // - SINE_TOLERANCE (1.05): Sine waves are mathematically bounded to [-1, 1], but floating-point precision and
    //   interpolation can cause ~5% overshoot.
    //
    // - POLYBLEP_TOLERANCE (1.1): Square/sawtooth/triangle waves use PolyBLEP/PolyBLAMP which intentionally overshoots
    //   by ~10% to smooth discontinuities and reduce aliasing.
    //
    // - PHASE_CONTINUITY_MULTIPLIER (3.0): When checking phase continuity across buffers, the expected maximum
    //   sample-to-sample difference is 2.0/samples_per_cycle. We allow 3x this value to account for PolyBLEP smoothing
    //   at discontinuities.

    const SINE_TOLERANCE: f64 = 1.05;
    const POLYBLEP_TOLERANCE: f64 = 1.1;
    const PHASE_CONTINUITY_MULTIPLIER: f64 = 3.0;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 1,
            buffer_size,
            current_sample: 0,
            channel_layout: ChannelLayout::Mono,
        }
    }

    fn test_oscillator<S: Sample>(waveform: Waveform, frequency: f64, buffer_size: usize) -> Vec<S> {
        let mut osc = OscillatorBlock::<S>::new(frequency, waveform, Some(42));
        let context = test_context(buffer_size);
        let inputs: [&[S]; 0] = [];
        let mut output = vec![S::ZERO; buffer_size];
        let mut outputs: [&mut [S]; 1] = [&mut output];
        osc.process(&inputs, &mut outputs, &[], &context);
        output
    }

    #[test]
    fn test_oscillator_input_output_counts_f32() {
        let osc = OscillatorBlock::<f32>::new(440.0, Waveform::Sine, None);
        assert_eq!(osc.input_count(), DEFAULT_GENERATOR_INPUT_COUNT);
        assert_eq!(osc.output_count(), DEFAULT_GENERATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_oscillator_input_output_counts_f64() {
        let osc = OscillatorBlock::<f64>::new(440.0, Waveform::Sine, None);
        assert_eq!(osc.input_count(), DEFAULT_GENERATOR_INPUT_COUNT);
        assert_eq!(osc.output_count(), DEFAULT_GENERATOR_OUTPUT_COUNT);
    }

    #[test]
    fn test_sine_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Sine, 440.0, 1024);
        let tolerance = SINE_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample >= -tolerance && sample <= tolerance,
                "Sine sample {} out of range (tolerance: {})",
                sample,
                tolerance
            );
        }
    }

    #[test]
    fn test_sine_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Sine, 440.0, 1024);
        for sample in output {
            assert!(
                sample >= -SINE_TOLERANCE && sample <= SINE_TOLERANCE,
                "Sine sample {} out of range (tolerance: {})",
                sample,
                SINE_TOLERANCE
            );
        }
    }

    #[test]
    fn test_square_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Square, 440.0, 1024);
        let tolerance = POLYBLEP_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample >= -tolerance && sample <= tolerance,
                "Square sample {} out of range (tolerance: {})",
                sample,
                tolerance
            );
        }
    }

    #[test]
    fn test_square_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Square, 440.0, 1024);
        for sample in output {
            assert!(
                sample >= -POLYBLEP_TOLERANCE && sample <= POLYBLEP_TOLERANCE,
                "Square sample {} out of range (tolerance: {})",
                sample,
                POLYBLEP_TOLERANCE
            );
        }
    }

    #[test]
    fn test_sawtooth_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Sawtooth, 440.0, 1024);
        let tolerance = POLYBLEP_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample >= -tolerance && sample <= tolerance,
                "Sawtooth sample {} out of range (tolerance: {})",
                sample,
                tolerance
            );
        }
    }

    #[test]
    fn test_sawtooth_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Sawtooth, 440.0, 1024);
        for sample in output {
            assert!(
                sample >= -POLYBLEP_TOLERANCE && sample <= POLYBLEP_TOLERANCE,
                "Sawtooth sample {} out of range (tolerance: {})",
                sample,
                POLYBLEP_TOLERANCE
            );
        }
    }

    #[test]
    fn test_triangle_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Triangle, 440.0, 1024);
        let tolerance = POLYBLEP_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample >= -tolerance && sample <= tolerance,
                "Triangle sample {} out of range (tolerance: {})",
                sample,
                tolerance
            );
        }
    }

    #[test]
    fn test_triangle_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Triangle, 440.0, 1024);
        for sample in output {
            assert!(
                sample >= -POLYBLEP_TOLERANCE && sample <= POLYBLEP_TOLERANCE,
                "Triangle sample {} out of range (tolerance: {})",
                sample,
                POLYBLEP_TOLERANCE
            );
        }
    }

    #[test]
    fn test_noise_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Noise, 440.0, 1024);
        for sample in output {
            assert!(sample >= -1.0 && sample <= 1.0, "Noise sample {} out of range", sample);
        }
    }

    #[test]
    fn test_noise_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Noise, 440.0, 1024);
        for sample in output {
            assert!(sample >= -1.0 && sample <= 1.0, "Noise sample {} out of range", sample);
        }
    }

    #[test]
    fn test_sine_produces_signal() {
        let output = test_oscillator::<f32>(Waveform::Sine, 440.0, 512);
        let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max > 0.5, "Sine should produce signal, max={}", max);
    }

    #[test]
    fn test_deterministic_with_seed_f32() {
        let output1 = test_oscillator::<f32>(Waveform::Sawtooth, 440.0, 256);
        let output2 = test_oscillator::<f32>(Waveform::Sawtooth, 440.0, 256);
        for (a, b) in output1.iter().zip(output2.iter()) {
            assert!((a - b).abs() < 1e-6, "Same seed should produce identical output");
        }
    }

    #[test]
    fn test_deterministic_with_seed_f64() {
        let output1 = test_oscillator::<f64>(Waveform::Sawtooth, 440.0, 256);
        let output2 = test_oscillator::<f64>(Waveform::Sawtooth, 440.0, 256);
        for (a, b) in output1.iter().zip(output2.iter()) {
            assert!((a - b).abs() < 1e-12, "Same seed should produce identical output");
        }
    }

    #[test]
    fn test_phase_continuity_across_buffers_f32() {
        let mut osc = OscillatorBlock::<f32>::new(440.0, Waveform::Sine, Some(42));
        let context = test_context(256);
        let inputs: [&[f32]; 0] = [];

        let mut buffer1 = vec![0.0f32; 256];
        let mut buffer2 = vec![0.0f32; 256];

        {
            let mut outputs: [&mut [f32]; 1] = [&mut buffer1];
            osc.process(&inputs, &mut outputs, &[], &context);
        }
        {
            let mut outputs: [&mut [f32]; 1] = [&mut buffer2];
            osc.process(&inputs, &mut outputs, &[], &context);
        }

        let last = buffer1[255];
        let first = buffer2[0];
        let diff = (last - first).abs();
        let samples_per_cycle = 44100.0 / 440.0;
        let expected_diff_per_sample = 2.0 / samples_per_cycle;
        assert!(
            diff < expected_diff_per_sample * PHASE_CONTINUITY_MULTIPLIER as f32,
            "Phase discontinuity detected: last={}, first={}, diff={}",
            last,
            first,
            diff
        );
    }

    #[test]
    fn test_phase_continuity_across_buffers_f64() {
        let mut osc = OscillatorBlock::<f64>::new(440.0, Waveform::Sine, Some(42));
        let context = test_context(256);
        let inputs: [&[f64]; 0] = [];

        let mut buffer1 = vec![0.0f64; 256];
        let mut buffer2 = vec![0.0f64; 256];

        {
            let mut outputs: [&mut [f64]; 1] = [&mut buffer1];
            osc.process(&inputs, &mut outputs, &[], &context);
        }
        {
            let mut outputs: [&mut [f64]; 1] = [&mut buffer2];
            osc.process(&inputs, &mut outputs, &[], &context);
        }

        let last = buffer1[255];
        let first = buffer2[0];
        let diff = (last - first).abs();
        let samples_per_cycle = 44100.0 / 440.0;
        let expected_diff_per_sample = 2.0 / samples_per_cycle;
        assert!(
            diff < expected_diff_per_sample * PHASE_CONTINUITY_MULTIPLIER,
            "Phase discontinuity detected: last={}, first={}, diff={}",
            last,
            first,
            diff
        );
    }

    #[test]
    fn test_frequency_accuracy_via_zero_crossings_f32() {
        let freq = 440.0;
        let sample_rate = 44100.0;
        let num_buffers = 10;
        let buffer_size = 512;
        let total_samples = num_buffers * buffer_size;

        let mut osc = OscillatorBlock::<f32>::new(freq, Waveform::Sine, Some(42));
        let context = test_context(buffer_size);
        let inputs: [&[f32]; 0] = [];

        let mut all_samples = Vec::with_capacity(total_samples);
        for _ in 0..num_buffers {
            let mut buffer = vec![0.0f32; buffer_size];
            {
                let mut outputs: [&mut [f32]; 1] = [&mut buffer];
                osc.process(&inputs, &mut outputs, &[], &context);
            }
            all_samples.extend(buffer);
        }

        let mut zero_crossings = 0;
        for i in 1..all_samples.len() {
            if (all_samples[i - 1] < 0.0 && all_samples[i] >= 0.0)
                || (all_samples[i - 1] >= 0.0 && all_samples[i] < 0.0)
            {
                zero_crossings += 1;
            }
        }

        let estimated_freq = (zero_crossings as f64 / 2.0) / (total_samples as f64 / sample_rate);
        let freq_error = (estimated_freq - freq as f64).abs();
        assert!(
            freq_error < 5.0,
            "Frequency error too large: expected {}, got {}, error={}",
            freq,
            estimated_freq,
            freq_error
        );
    }

    #[test]
    fn test_midi_frequency_override_f32() {
        let mut osc = OscillatorBlock::<f32>::new(440.0, Waveform::Sine, Some(42));
        osc.set_midi_frequency(880.0);

        let context = test_context(512);
        let inputs: [&[f32]; 0] = [];
        let mut output = vec![0.0f32; 512];
        let mut outputs: [&mut [f32]; 1] = [&mut output];
        osc.process(&inputs, &mut outputs, &[], &context);

        let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max > 0.5, "Should produce signal with MIDI frequency");
    }

    #[test]
    fn test_clear_midi_frequency_f32() {
        let mut osc = OscillatorBlock::<f32>::new(440.0, Waveform::Sine, Some(42));
        osc.set_midi_frequency(880.0);
        osc.clear_midi_frequency();

        let context = test_context(512);
        let inputs: [&[f32]; 0] = [];
        let mut output = vec![0.0f32; 512];
        let mut outputs: [&mut [f32]; 1] = [&mut output];
        osc.process(&inputs, &mut outputs, &[], &context);

        let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max > 0.5, "Should produce signal after clearing MIDI frequency");
    }

    #[test]
    fn test_set_waveform_f32() {
        let mut osc = OscillatorBlock::<f32>::new(440.0, Waveform::Sine, Some(42));
        osc.set_waveform(Waveform::Square);

        let context = test_context(512);
        let inputs: [&[f32]; 0] = [];
        let mut output = vec![0.0f32; 512];
        let mut outputs: [&mut [f32]; 1] = [&mut output];
        osc.process(&inputs, &mut outputs, &[], &context);

        let near_one = output.iter().filter(|&&x| (x.abs() - 1.0).abs() < 0.2).count();
        assert!(
            near_one > output.len() / 4,
            "Square wave should have many samples near +/-1"
        );
    }

    #[test]
    fn test_low_frequency_f32() {
        let output = test_oscillator::<f32>(Waveform::Sine, 20.0, 4096);
        let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max > 0.3, "Low frequency should still produce signal");
    }

    #[test]
    fn test_high_frequency_f32() {
        let output = test_oscillator::<f32>(Waveform::Sine, 15000.0, 512);
        let max = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max > 0.5, "High frequency should produce signal");
    }

    #[test]
    fn test_nyquist_frequency_f32() {
        let output = test_oscillator::<f32>(Waveform::Sine, 22000.0, 512);
        let tolerance = POLYBLEP_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample.abs() <= tolerance,
                "Near-Nyquist frequency should not produce artifacts above tolerance ({})",
                tolerance
            );
        }
    }

    #[test]
    fn test_pulse_output_range_f32() {
        let output = test_oscillator::<f32>(Waveform::Pulse, 440.0, 1024);
        let tolerance = POLYBLEP_TOLERANCE as f32;
        for sample in output {
            assert!(
                sample >= -tolerance && sample <= tolerance,
                "Pulse sample {} out of range (tolerance: {})",
                sample,
                tolerance
            );
        }
    }

    #[test]
    fn test_pulse_output_range_f64() {
        let output = test_oscillator::<f64>(Waveform::Pulse, 440.0, 1024);
        for sample in output {
            assert!(
                sample >= -POLYBLEP_TOLERANCE && sample <= POLYBLEP_TOLERANCE,
                "Pulse sample {} out of range (tolerance: {})",
                sample,
                POLYBLEP_TOLERANCE
            );
        }
    }

    #[test]
    fn test_noise_varies_f32() {
        let output = test_oscillator::<f32>(Waveform::Noise, 440.0, 256);
        let first = output[0];
        let varies = output.iter().any(|&x| (x - first).abs() > 0.01);
        assert!(varies, "Noise should produce varying values");
    }

    #[test]
    fn test_noise_varies_f64() {
        let output = test_oscillator::<f64>(Waveform::Noise, 440.0, 256);
        let first = output[0];
        let varies = output.iter().any(|&x| (x - first).abs() > 0.01);
        assert!(varies, "Noise should produce varying values");
    }
}
