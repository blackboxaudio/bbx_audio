use crate::{
    block::{Block, DEFAULT_MODULATOR_INPUT_COUNT, DEFAULT_MODULATOR_OUTPUT_COUNT},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// ADSR envelope stages.
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

    /// Create an `EnvelopeBlock` with given ADSR parameters.
    /// Times are in seconds, sustain is a level from 0.0 to 1.0.
    pub fn new(attack: S, decay: S, sustain: S, release: S) -> Self {
        Self {
            attack: Parameter::Constant(attack),
            decay: Parameter::Constant(decay),
            sustain: Parameter::Constant(sustain),
            release: Parameter::Constant(release),
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
                    if self.level <= 0.0 {
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
