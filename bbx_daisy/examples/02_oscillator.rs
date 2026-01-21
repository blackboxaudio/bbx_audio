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

use core::f32::consts::PI;

use bbx_daisy::{
    FrameBuffer,
    audio::{self, AudioCallback, BLOCK_SIZE},
    context::DEFAULT_SAMPLE_RATE,
};
use cortex_m_rt::entry;
use panic_halt as _;

const FREQUENCY: f32 = 440.0;
const AMPLITUDE: f32 = 0.5;

static mut PHASE: f32 = 0.0;
static mut PHASE_INCREMENT: f32 = 0.0;

fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
    unsafe {
        for i in 0..BLOCK_SIZE {
            let sample = libm::sinf(PHASE * 2.0 * PI) * AMPLITUDE;

            output.set_frame(i, sample, sample);

            PHASE += PHASE_INCREMENT;
            if PHASE >= 1.0 {
                PHASE -= 1.0;
            }
        }
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        PHASE_INCREMENT = FREQUENCY / DEFAULT_SAMPLE_RATE;
    }

    audio::set_callback(audio_callback);

    let mut audio_interface = audio::default_audio();
    audio_interface.init();
    audio_interface.start();

    loop {
        cortex_m::asm::wfi();
    }
}
