//! # 03_filter_knob - Filter with Knob Control Example
//!
//! A sawtooth oscillator through a low-pass filter with knob-controlled cutoff.
//!
//! ## Hardware
//!
//! - Daisy Seed with potentiometer connected to D21 (PC4)
//! - Audio output connected to speakers/headphones
//!
//! ## Building & Flashing
//!
//! ```bash
//! cargo build --example 03_filter_knob --target thumbv7em-none-eabihf --release
//! probe-rs run --chip STM32H750VBTx target/thumbv7em-none-eabihf/release/examples/03_filter_knob
//! ```

#![no_std]
#![no_main]

use core::f32::consts::PI;

use bbx_daisy::{
    FrameBuffer,
    audio::{self, AudioCallback, BLOCK_SIZE},
    context::DEFAULT_SAMPLE_RATE,
    peripherals::adc::Knob,
};
use cortex_m_rt::entry;
use panic_halt as _;

const FREQUENCY: f32 = 110.0;
const AMPLITUDE: f32 = 0.5;

static mut OSC_PHASE: f32 = 0.0;
static mut PHASE_INCREMENT: f32 = 0.0;

static mut FILTER_STATE: f32 = 0.0;
static mut FILTER_CUTOFF: f32 = 1000.0;

fn calculate_filter_coefficient(cutoff: f32, sample_rate: f32) -> f32 {
    let omega = 2.0 * PI * cutoff / sample_rate;
    omega / (omega + 1.0)
}

fn audio_callback(_input: &FrameBuffer<BLOCK_SIZE>, output: &mut FrameBuffer<BLOCK_SIZE>) {
    unsafe {
        let coefficient = calculate_filter_coefficient(FILTER_CUTOFF, DEFAULT_SAMPLE_RATE);

        for i in 0..BLOCK_SIZE {
            let saw = (OSC_PHASE * 2.0 - 1.0) * AMPLITUDE;

            FILTER_STATE = FILTER_STATE + coefficient * (saw - FILTER_STATE);
            let filtered = FILTER_STATE;

            output.set_frame(i, filtered, filtered);

            OSC_PHASE += PHASE_INCREMENT;
            if OSC_PHASE >= 1.0 {
                OSC_PHASE -= 1.0;
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

    let mut knob = Knob::new(0.1);

    loop {
        // Read ADC (simulated - in real hardware, read from ADC peripheral)
        // let raw_value = adc.read(&mut knob_pin);
        let raw_value: u16 = 2048; // Simulated middle position

        let knob_value = knob.process_u12(raw_value);

        unsafe {
            FILTER_CUTOFF = knob.map_exponential(100.0, 10000.0, 2.0);
        }

        cortex_m::asm::delay(48_000);
    }
}
