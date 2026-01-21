//! # 04_simple_synth - Complete Synthesis Chain Example
//!
//! A complete synthesizer voice with:
//! - Sawtooth oscillator
//! - Low-pass filter with envelope modulation
//! - ADSR amplitude envelope
//!
//! This example demonstrates a complex audio processor with multiple
//! interacting components (oscillator, filter, envelopes).
//!
//! ## Hardware
//!
//! - Daisy Seed (any variant)
//! - Audio output connected to speakers/headphones
//!
//! ## Building & Flashing
//!
//! ```bash
//! cargo build --example 04_simple_synth --target thumbv7em-none-eabihf --release
//! probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/04_simple_synth
//! ```

#![no_std]
#![no_main]

use bbx_daisy::{bbx_daisy_audio, prelude::*};

const BASE_FREQUENCY: f32 = 220.0;

#[derive(Clone, Copy, PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

struct Envelope {
    stage: EnvelopeStage,
    level: f32,
    attack_rate: f32,
    decay_rate: f32,
    sustain_level: f32,
    release_rate: f32,
}

impl Envelope {
    const fn new() -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            level: 0.0,
            attack_rate: 0.001,
            decay_rate: 0.0001,
            sustain_level: 0.7,
            release_rate: 0.0005,
        }
    }

    fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    fn release(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    fn process(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }
            EnvelopeStage::Attack => {
                self.level += self.attack_rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                self.level -= self.decay_rate;
                if self.level <= self.sustain_level {
                    self.level = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {}
            EnvelopeStage::Release => {
                self.level -= self.release_rate;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
        self.level
    }
}

struct LowPassFilter {
    state: f32,
    cutoff: f32,
}

impl LowPassFilter {
    const fn new() -> Self {
        Self {
            state: 0.0,
            cutoff: 1000.0,
        }
    }

    fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = if cutoff < 20.0 {
            20.0
        } else if cutoff > 20000.0 {
            20000.0
        } else {
            cutoff
        };
    }

    fn process(&mut self, input: f32) -> f32 {
        let omega = 2.0 * PI * self.cutoff / DEFAULT_SAMPLE_RATE;
        let alpha = omega / (omega + 1.0);
        self.state = self.state + alpha * (input - self.state);
        self.state
    }
}

struct Oscillator {
    phase: f32,
    frequency: f32,
}

impl Oscillator {
    const fn new() -> Self {
        Self {
            phase: 0.0,
            frequency: BASE_FREQUENCY,
        }
    }

    fn saw(&mut self) -> f32 {
        let output = self.phase * 2.0 - 1.0;
        self.phase += self.frequency / DEFAULT_SAMPLE_RATE;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        output
    }
}

struct Synth {
    osc: Oscillator,
    filter: LowPassFilter,
    amp_env: Envelope,
    filter_env: Envelope,
    base_cutoff: f32,
    env_amount: f32,
}

impl Synth {
    const fn new() -> Self {
        Self {
            osc: Oscillator::new(),
            filter: LowPassFilter::new(),
            amp_env: Envelope::new(),
            filter_env: Envelope::new(),
            base_cutoff: 800.0,
            env_amount: 3000.0,
        }
    }

    fn note_on(&mut self) {
        self.amp_env.trigger();
        self.filter_env.trigger();
    }

    fn process_sample(&mut self) -> f32 {
        let osc_out = self.osc.saw();

        let filter_env = self.filter_env.process();
        let cutoff = self.base_cutoff + self.env_amount * filter_env;
        self.filter.set_cutoff(cutoff);
        let filtered = self.filter.process(osc_out);

        let amp_env = self.amp_env.process();
        filtered * amp_env * 0.5
    }
}

impl AudioProcessor for Synth {
    fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        for i in 0..BLOCK_SIZE {
            let sample = self.process_sample();
            output.set_frame(i, sample, sample);
        }
    }

    fn prepare(&mut self, _sample_rate: f32) {
        self.note_on();
    }
}

bbx_daisy_audio!(Synth, Synth::new());
