//! LFO-modulated filter sweep (classic wah effect).
//!
//! Demonstrates real-time filter cutoff modulation using an LFO.
//! A sawtooth oscillator is filtered with a resonant low-pass filter
//! whose cutoff sweeps up and down.
//!
//! Signal chain: Oscillator(110Hz, Saw) -> LowPassFilter(Q:4) -> Gain -> Output
//! Modulation: LFO(0.5Hz, depth:4000) modulates filter cutoff

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Rich harmonic source for filter to work on
    let oscillator = builder.add_oscillator(110.0, Waveform::Sawtooth, None);

    // Resonant low-pass filter with high Q for pronounced sweep
    let filter = builder.add_low_pass_filter(800.0, 4.0);

    // LFO to sweep the cutoff frequency
    let lfo = builder.add_lfo(0.5, 3000.0, None);

    // Output gain
    let gain = builder.add_gain(-9.0, None);

    // Build signal chain
    builder.connect(oscillator, 0, filter, 0);
    builder.connect(filter, 0, gain, 0);
    builder.modulate(lfo, filter, "cutoff");

    builder.build()
}

fn main() {
    println!("Filter Modulation Demo - Classic wah/sweep effect");
    let player = Player::from_graph(create_graph());
    player.play(None);
}
