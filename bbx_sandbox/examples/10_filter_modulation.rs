//! LFO-modulated filter sweep (classic wah effect).
//!
//! Demonstrates real-time filter cutoff modulation using an LFO.
//! A sawtooth oscillator is filtered with a resonant low-pass filter
//! whose cutoff sweeps up and down.
//!
//! Signal chain: Oscillator(110Hz, Saw) -> LowPassFilter(Q:4) -> Gain -> Output
//! Modulation: LFO(0.5Hz, depth:4000) modulates filter cutoff

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, LowPassFilterBlock, OscillatorBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Rich harmonic source for filter to work on
    let oscillator = builder.add(OscillatorBlock::new(110.0, Waveform::Sawtooth, None));

    // Resonant low-pass filter with high Q for pronounced sweep
    let filter = builder.add(LowPassFilterBlock::new(800.0, 4.0));

    // LFO to sweep the cutoff frequency
    let lfo = builder.add(LfoBlock::new(0.5, 3000.0, Waveform::Sine, None));

    // Output gain
    let gain = builder.add(GainBlock::new(-9.0, None));

    // Build signal chain
    builder.connect(oscillator, 0, filter, 0);
    builder.connect(filter, 0, gain, 0);
    builder.modulate(lfo, filter, "cutoff");

    builder.build()
}

fn main() {
    println!("Filter Modulation Demo - Classic wah/sweep effect");
    println!("Press Ctrl+C to stop.");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).unwrap();

    let player = Player::new(create_graph()).unwrap();
    let _handle = player.play().unwrap();

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }
}
