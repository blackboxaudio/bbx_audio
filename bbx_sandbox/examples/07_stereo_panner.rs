//! Evolving quartal pad with stereo panning.
//!
//! Creates an atmospheric ambient pad using quartal harmony (stacked perfect fourths)
//! with an open voicing spread across octaves. Features slow LFO modulation of stereo
//! panner positions plus filter cutoff modulation for evolving timbral movement.
//! Slight detuning on some oscillators creates organic beating.

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Layer 1: Sub-bass anchor (55 Hz, A1) - foundation
    let osc1 = builder.add_oscillator(55.0, Waveform::Sine, None);
    let gain1 = builder.add_gain(-6.0, None);
    let pan1 = builder.add_panner_stereo(0.0);
    let lfo1 = builder.add_lfo(0.02, 40.0, None);

    builder.connect(osc1, 0, gain1, 0);
    builder.connect(gain1, 0, pan1, 0);
    builder.modulate(lfo1, pan1, "position");

    // Layer 2: Root (110.3 Hz, A2 +3 cents) - warm body
    let osc2 = builder.add_oscillator(110.3, Waveform::Triangle, None);
    let gain2 = builder.add_gain(-9.0, None);
    let pan2 = builder.add_panner_stereo(0.0);
    let lfo2 = builder.add_lfo(0.04, 60.0, None);

    builder.connect(osc2, 0, gain2, 0);
    builder.connect(gain2, 0, pan2, 0);
    builder.modulate(lfo2, pan2, "position");

    // Layer 3: 1st fourth (293.0 Hz, D4 -2 cents) - mid clarity
    let osc3 = builder.add_oscillator(293.0, Waveform::Triangle, None);
    let gain3 = builder.add_gain(-10.0, None);
    let pan3 = builder.add_panner_stereo(0.0);
    let lfo3 = builder.add_lfo(0.06, 70.0, None);

    builder.connect(osc3, 0, gain3, 0);
    builder.connect(gain3, 0, pan3, 0);
    builder.modulate(lfo3, pan3, "position");

    // Layer 4: 2nd fourth (392.5 Hz, G4 +3 cents) - presence
    let osc4 = builder.add_oscillator(392.5, Waveform::Triangle, None);
    let gain4 = builder.add_gain(-10.0, None);
    let pan4 = builder.add_panner_stereo(0.0);
    let lfo4 = builder.add_lfo(0.09, 80.0, None);

    builder.connect(osc4, 0, gain4, 0);
    builder.connect(gain4, 0, pan4, 0);
    builder.modulate(lfo4, pan4, "position");

    // Layer 5: 3rd fourth (523.0 Hz, C5) - evolving texture with filter modulation
    let osc5 = builder.add_oscillator(523.0, Waveform::Sawtooth, None);
    let lpf = builder.add_low_pass_filter(1200.0, 1.2);
    let filter_lfo = builder.add_lfo(0.04, 600.0, None);
    let gain5 = builder.add_gain(-12.0, None);
    let pan5 = builder.add_panner_stereo(0.0);
    let pan_lfo5 = builder.add_lfo(0.11, 85.0, None);

    builder.connect(osc5, 0, lpf, 0);
    builder.modulate(filter_lfo, lpf, "cutoff");
    builder.connect(lpf, 0, gain5, 0);
    builder.connect(gain5, 0, pan5, 0);
    builder.modulate(pan_lfo5, pan5, "position");

    // Layer 6: 4th fourth (697.0 Hz, F5 -2 cents) - shimmer
    let osc6 = builder.add_oscillator(697.0, Waveform::Sine, None);
    let gain6 = builder.add_gain(-15.0, None);
    let pan6 = builder.add_panner_stereo(0.0);
    let lfo6 = builder.add_lfo(0.07, 100.0, None);

    builder.connect(osc6, 0, gain6, 0);
    builder.connect(gain6, 0, pan6, 0);
    builder.modulate(lfo6, pan6, "position");

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(Some(60));
}
