//! Waveform oscillator block.

use bbx_core::random::XorShiftRng;

#[cfg(feature = "simd")]
use crate::polyblep::PolyBlepState;
#[cfg(feature = "simd")]
use crate::sample::SIMD_LANES;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd_antialiased;
#[cfg(feature = "simd")]
use crate::waveform::generate_waveform_samples_simd_generic;
use crate::{
    block::{Block, DEFAULT_GENERATOR_INPUT_COUNT, DEFAULT_GENERATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    polyblep::AntiAliasingMode,
    sample::Sample,
    waveform::{Waveform, process_waveform_scalar, process_waveform_scalar_antialiased},
};

/// A waveform oscillator for generating audio signals.
///
/// Supports standard waveforms (sine, square, sawtooth, triangle, pulse, noise).
/// Frequency can be controlled via parameter modulation or MIDI note messages.
///
/// Anti-aliasing is enabled by default using 4th-order PolyBLEP/PolyBLAMP,
/// which reduces aliasing artifacts from discontinuous waveforms at high frequencies.
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
    antialiasing: AntiAliasingMode,
    #[cfg(feature = "simd")]
    #[allow(dead_code)] // Reserved for future cross-chunk correction optimization
    polyblep_state: PolyBlepState<S>,
}

impl<S: Sample> OscillatorBlock<S> {
    /// Create a new oscillator with the given frequency and waveform.
    ///
    /// Anti-aliasing is enabled by default.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Base frequency in Hz
    /// * `waveform` - The waveform shape to generate
    /// * `seed` - Optional RNG seed for noise waveform
    pub fn new(frequency: S, waveform: Waveform, seed: Option<u64>) -> Self {
        Self {
            frequency: Parameter::Constant(frequency),
            pitch_offset: Parameter::Constant(S::ZERO),
            base_frequency: frequency,
            midi_frequency: None,
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
            antialiasing: AntiAliasingMode::default(),
            #[cfg(feature = "simd")]
            polyblep_state: PolyBlepState::new(),
        }
    }

    /// Create a new oscillator with explicit anti-aliasing mode.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Base frequency in Hz
    /// * `waveform` - The waveform shape to generate
    /// * `antialiasing` - Anti-aliasing mode (None or PolyBlep)
    /// * `seed` - Optional RNG seed for noise waveform
    pub fn with_antialiasing(
        frequency: S,
        waveform: Waveform,
        antialiasing: AntiAliasingMode,
        seed: Option<u64>,
    ) -> Self {
        Self {
            frequency: Parameter::Constant(frequency),
            pitch_offset: Parameter::Constant(S::ZERO),
            base_frequency: frequency,
            midi_frequency: None,
            phase: 0.0,
            waveform,
            rng: XorShiftRng::new(seed.unwrap_or_default()),
            antialiasing,
            #[cfg(feature = "simd")]
            polyblep_state: PolyBlepState::new(),
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

    /// Set the anti-aliasing mode.
    pub fn set_antialiasing(&mut self, mode: AntiAliasingMode) {
        self.antialiasing = mode;
    }

    /// Get the current anti-aliasing mode.
    pub fn antialiasing(&self) -> AntiAliasingMode {
        self.antialiasing
    }
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let base = self.midi_frequency.unwrap_or(self.base_frequency);

        let freq_hz = match &self.frequency {
            Parameter::Constant(f) => self.midi_frequency.unwrap_or(*f),
            Parameter::Modulated(block_id) => {
                // Add modulation to base (allows vibrato on top of MIDI note)
                // Safe lookup to prevent panic in audio thread
                let mod_value = modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO);
                base + mod_value
            }
        };

        let pitch_offset_semitones = match &self.pitch_offset {
            Parameter::Constant(offset) => *offset,
            Parameter::Modulated(block_id) => {
                // Safe lookup to prevent panic in audio thread
                modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO)
            }
        };

        // Convert semitones to frequency multiplier: 2^(semitones/12)
        let freq = if pitch_offset_semitones != S::ZERO {
            let multiplier = S::from_f64(2.0f64.powf(pitch_offset_semitones.to_f64() / 12.0));
            freq_hz * multiplier
        } else {
            freq_hz
        };

        let phase_increment = freq.to_f64() / context.sample_rate * std::f64::consts::TAU;
        let use_antialiasing = matches!(self.antialiasing, AntiAliasingMode::PolyBlep);

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
                let two_pi = S::simd_splat(S::from_f64(std::f64::consts::TAU));
                let inv_two_pi = S::simd_splat(S::from_f64(1.0 / std::f64::consts::TAU));

                // Normalized phase increment for PolyBLEP calculations
                let phase_inc_normalized = S::from_f64(phase_increment / std::f64::consts::TAU);

                for chunk_idx in 0..chunks {
                    let samples = if use_antialiasing {
                        // Convert SIMD phases to normalized Sample array for PolyBLEP
                        let phases_array = S::simd_to_array(phases);
                        let inv_tau = S::from_f64(1.0 / std::f64::consts::TAU);
                        let phases_normalized: [S; SIMD_LANES] = [
                            (phases_array[0] * inv_tau) - S::from_f64((phases_array[0].to_f64() / std::f64::consts::TAU).floor()),
                            (phases_array[1] * inv_tau) - S::from_f64((phases_array[1].to_f64() / std::f64::consts::TAU).floor()),
                            (phases_array[2] * inv_tau) - S::from_f64((phases_array[2].to_f64() / std::f64::consts::TAU).floor()),
                            (phases_array[3] * inv_tau) - S::from_f64((phases_array[3].to_f64() / std::f64::consts::TAU).floor()),
                        ];

                        let result = generate_waveform_samples_simd_antialiased::<S>(
                            self.waveform,
                            phases,
                            phases_normalized,
                            phase_inc_normalized,
                            duty,
                            two_pi,
                            inv_two_pi,
                        );

                        result
                    } else {
                        generate_waveform_samples_simd_generic::<S>(self.waveform, phases, duty, two_pi, inv_two_pi)
                    };

                    if let Some(samples) = samples {
                        let base = chunk_idx * SIMD_LANES;
                        outputs[0][base..base + SIMD_LANES].copy_from_slice(&samples);
                    }

                    phases = phases + chunk_inc_simd;
                }

                self.phase += chunk_phase_step * chunks as f64;
                self.phase = self.phase.rem_euclid(std::f64::consts::TAU);

                // Handle remainder samples
                if use_antialiasing {
                    process_waveform_scalar_antialiased(
                        &mut outputs[0][remainder_start..],
                        self.waveform,
                        &mut self.phase,
                        phase_increment,
                        &mut self.rng,
                        1.0,
                    );
                } else {
                    process_waveform_scalar(
                        &mut outputs[0][remainder_start..],
                        self.waveform,
                        &mut self.phase,
                        phase_increment,
                        &mut self.rng,
                        1.0,
                    );
                }
            } else {
                // Noise waveform doesn't need anti-aliasing
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
            if use_antialiasing {
                process_waveform_scalar_antialiased(
                    outputs[0],
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
}
