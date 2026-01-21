//! # 04_simple_synth - Complete Synthesis Chain Example
//!
//! A complete synthesizer voice with:
//! - Sawtooth oscillator
//! - Low-pass filter with envelope modulation
//! - ADSR amplitude envelope
//! - Button-triggered note on/off
//!
//! ## Hardware
//!
//! - Daisy Seed
//! - Button connected to D0 (PB12) with pull-up
//! - Potentiometer on D21 (PC4) for filter cutoff
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

use core::f32::consts::PI;

use cortex_m_rt::entry;
use panic_halt as _;

use bbx_daisy::audio::{self, AudioCallback, BLOCK_SIZE};
use bbx_daisy::context::DEFAULT_SAMPLE_RATE;
use bbx_daisy::FrameBuffer;

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
        self.cutoff = cutoff.clamp(20.0, 20000.0);
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

    fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(20.0, 20000.0);
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

static mut SYNTH: Synth = Synth::new();

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
            base_cutoff: 500.0,
            env_amount: 2000.0,
        }
    }

    fn note_on(&mut self, frequency: f32) {
        self.osc.set_frequency(frequency);
        self.amp_env.trigger();
        self.filter_env.trigger();
    }

    fn note_off(&mut self) {
        self.amp_env.release();
        self.filter_env.release();
    }

    fn process(&mut self) -> f32 {
        let osc_out = self.osc.saw();

        let filter_env = self.filter_env.process();
        let cutoff = self.base_cutoff + self.env_amount * filter_env;
        self.filter.set_cutoff(cutoff);
        let filtered = self.filter.process(osc_out);

        let amp_env = self.amp_env.process();
        filtered * amp_env * 0.5
    }
}

fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
    unsafe {
        let synth = core::ptr::addr_of_mut!(SYNTH);
        for i in 0..BLOCK_SIZE {
            let sample = (*synth).process();
            output.set_frame(i, sample, sample);
        }
    }
}

#[entry]
fn main() -> ! {
    audio::set_callback(audio_callback);

    let mut audio_interface = audio::default_audio();
    audio_interface.init();
    audio_interface.start();

    let mut note_playing = false;
    let mut button_was_pressed = false;

    unsafe {
        let synth = core::ptr::addr_of_mut!(SYNTH);
        (*synth).base_cutoff = 800.0;
        (*synth).env_amount = 3000.0;
    }

    loop {
        let button_pressed = false;

        if button_pressed && !button_was_pressed {
            unsafe {
                let synth = core::ptr::addr_of_mut!(SYNTH);
                if note_playing {
                    (*synth).note_off();
                    note_playing = false;
                } else {
                    (*synth).note_on(BASE_FREQUENCY);
                    note_playing = true;
                }
            }
        }
        button_was_pressed = button_pressed;

        cortex_m::asm::delay(48_000);
    }
}
