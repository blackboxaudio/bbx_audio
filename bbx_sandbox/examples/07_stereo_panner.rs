//! Stereo panning drone example.
//!
//! Creates a hypnotic ambient drone using multiple layered oscillators
//! with slow LFO modulation of stereo panner positions. Each layer moves
//! independently across the stereo field at different rates.

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Layer 1: Sub bass (55 Hz, A1) - foundation, subtle pan movement
    let osc1 = builder.add_oscillator(55.0, Waveform::Sine, None);
    let gain1 = builder.add_gain(-6.0, None);
    let pan1 = builder.add_panner_stereo(0.0);
    let lfo1 = builder.add_lfo(0.03, 40.0, None);

    builder.connect(osc1, 0, gain1, 0);
    builder.connect(gain1, 0, pan1, 0);
    builder.modulate(lfo1, pan1, "position");

    // Layer 2: Low-mid (82.5 Hz, E2 - perfect fifth) - warmth
    let osc2 = builder.add_oscillator(82.5, Waveform::Sine, None);
    let gain2 = builder.add_gain(-9.0, None);
    let pan2 = builder.add_panner_stereo(0.0);
    let lfo2 = builder.add_lfo(0.05, 60.0, None);

    builder.connect(osc2, 0, gain2, 0);
    builder.connect(gain2, 0, pan2, 0);
    builder.modulate(lfo2, pan2, "position");

    // Layer 3: Mid (110 Hz, A2 - octave) - body
    let osc3 = builder.add_oscillator(110.0, Waveform::Sine, None);
    let gain3 = builder.add_gain(-9.0, None);
    let pan3 = builder.add_panner_stereo(0.0);
    let lfo3 = builder.add_lfo(0.08, 80.0, None);

    builder.connect(osc3, 0, gain3, 0);
    builder.connect(gain3, 0, pan3, 0);
    builder.modulate(lfo3, pan3, "position");

    // Layer 4: Upper-mid (220 Hz, A3) with filtering - texture
    let osc4 = builder.add_oscillator(220.0, Waveform::Sawtooth, None);
    let lpf = builder.add_low_pass_filter(800.0, 1.5);
    let gain4 = builder.add_gain(-12.0, None);
    let pan4 = builder.add_panner_stereo(0.0);
    let lfo4 = builder.add_lfo(0.12, 90.0, None);

    builder.connect(osc4, 0, lpf, 0);
    builder.connect(lpf, 0, gain4, 0);
    builder.connect(gain4, 0, pan4, 0);
    builder.modulate(lfo4, pan4, "position");

    // Layer 5: High (330 Hz, E4 - octave + fifth) - ethereal
    let osc5 = builder.add_oscillator(330.0, Waveform::Sine, None);
    let gain5 = builder.add_gain(-15.0, None);
    let pan5 = builder.add_panner_stereo(0.0);
    let lfo5 = builder.add_lfo(0.07, 100.0, None);

    builder.connect(osc5, 0, gain5, 0);
    builder.connect(gain5, 0, pan5, 0);
    builder.modulate(lfo5, pan5, "position");

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(Some(60));
}
