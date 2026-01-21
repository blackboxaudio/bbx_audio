//! # 02_oscillator - Sine Wave Output Example
//!
//! Generates a 440 Hz sine wave through the audio output.
//!
//! ## Hardware
//!
//! - Daisy Seed (any variant)
//! - Audio output connected to speakers/headphones
//!
//! ## Building & Flashing
//!
//! ```bash
//! cargo build --example 02_oscillator --target thumbv7em-none-eabihf --release
//! probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/02_oscillator
//! ```

#![no_std]
#![no_main]

use bbx_daisy::{bbx_daisy_audio, prelude::*};

const FREQUENCY: f32 = 440.0;
const AMPLITUDE: f32 = 0.5;

struct SineOscillator {
    phase: f32,
    phase_inc: f32,
}

impl SineOscillator {
    fn new(frequency: f32) -> Self {
        Self {
            phase: 0.0,
            phase_inc: frequency / DEFAULT_SAMPLE_RATE,
        }
    }
}

impl AudioProcessor for SineOscillator {
    fn process(&mut self, _input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
        for i in 0..BLOCK_SIZE {
            let sample = sinf(self.phase * 2.0 * PI) * AMPLITUDE;

            output.set_frame(i, sample, sample);

            self.phase += self.phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}

bbx_daisy_audio!(SineOscillator, SineOscillator::new(FREQUENCY));
