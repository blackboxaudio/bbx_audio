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

use bbx_daisy::{
    bbx_daisy_audio,
    dsp::{ChannelLayout, block::Block, blocks::EnvelopeBlock, context::DspContext},
    prelude::*,
};

const BASE_FREQUENCY: f32 = 220.0;

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
    amp_env: EnvelopeBlock<f32>,
    filter_env: EnvelopeBlock<f32>,
    amp_env_buffer: [f32; BLOCK_SIZE],
    filter_env_buffer: [f32; BLOCK_SIZE],
    base_cutoff: f32,
    env_amount: f32,
}

impl Synth {
    fn new() -> Self {
        Self {
            osc: Oscillator::new(),
            filter: LowPassFilter::new(),
            amp_env: EnvelopeBlock::new(0.01, 0.1, 0.7, 0.3),
            filter_env: EnvelopeBlock::new(0.01, 0.1, 0.7, 0.3),
            amp_env_buffer: [0.0; BLOCK_SIZE],
            filter_env_buffer: [0.0; BLOCK_SIZE],
            base_cutoff: 800.0,
            env_amount: 3000.0,
        }
    }

    fn note_on(&mut self) {
        self.amp_env.note_on();
        self.filter_env.note_on();
    }
}

impl AudioProcessor for Synth {
    fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        let context = DspContext {
            sample_rate: DEFAULT_SAMPLE_RATE as f64,
            num_channels: 2,
            buffer_size: BLOCK_SIZE,
            current_sample: 0,
            channel_layout: ChannelLayout::Stereo,
        };

        let inputs: [&[f32]; 0] = [];
        {
            let mut outputs: [&mut [f32]; 1] = [&mut self.amp_env_buffer];
            self.amp_env.process(&inputs, &mut outputs, &[], &context);
        }
        {
            let mut outputs: [&mut [f32]; 1] = [&mut self.filter_env_buffer];
            self.filter_env.process(&inputs, &mut outputs, &[], &context);
        }

        for i in 0..BLOCK_SIZE {
            let osc_out = self.osc.saw();

            let cutoff = self.base_cutoff + self.env_amount * self.filter_env_buffer[i];
            self.filter.set_cutoff(cutoff);
            let filtered = self.filter.process(osc_out);

            let sample = filtered * self.amp_env_buffer[i] * 0.5;
            output.set_frame(i, sample, sample);
        }
    }

    fn prepare(&mut self, _sample_rate: f32) {
        self.note_on();
    }
}

bbx_daisy_audio!(Synth, Synth::new());
