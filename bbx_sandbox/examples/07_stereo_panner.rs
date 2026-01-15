//! Polygonia-style evolving dark pad with stereo panning.
//!
//! Creates a hypnotic techno pad using an Am9 voicing with Phrygian color tones.
//! Features a B-C semitone cluster for characteristic shimmer, and an added F
//! for modal darkness. Prime-number-based LFO rates create polyrhythmic pan
//! movement that never quite repeats - evolving and hypnotic.

use std::time::Duration;

use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Layer 1: Sub root (A1, 55 Hz) - dark foundation
    let osc1 = builder.add(OscillatorBlock::new(55.0, Waveform::Sine, None));
    let gain1 = builder.add(GainBlock::new(-6.0, None));
    let pan1 = builder.add(PannerBlock::new(0.0));
    let lfo1 = builder.add(LfoBlock::new(0.017, 25.0, Waveform::Sine, None));

    builder.connect(osc1, 0, gain1, 0);
    builder.connect(gain1, 0, pan1, 0);
    builder.modulate(lfo1, pan1, "position");

    // Layer 2: P5 anchor (E2, 82.4 Hz) - weight
    let osc2 = builder.add(OscillatorBlock::new(82.4, Waveform::Triangle, None));
    let gain2 = builder.add(GainBlock::new(-9.0, None));
    let pan2 = builder.add(PannerBlock::new(0.0));
    let lfo2 = builder.add(LfoBlock::new(0.023, 45.0, Waveform::Sine, None));

    builder.connect(osc2, 0, gain2, 0);
    builder.connect(gain2, 0, pan2, 0);
    builder.modulate(lfo2, pan2, "position");

    // Layer 3: m7 (G3, 196.0 Hz) - minor color
    let osc3 = builder.add(OscillatorBlock::new(196.0, Waveform::Triangle, None));
    let gain3 = builder.add(GainBlock::new(-10.0, None));
    let pan3 = builder.add(PannerBlock::new(0.0));
    let lfo3 = builder.add(LfoBlock::new(0.031, 55.0, Waveform::Sine, None));

    builder.connect(osc3, 0, gain3, 0);
    builder.connect(gain3, 0, pan3, 0);
    builder.modulate(lfo3, pan3, "position");

    // Layer 4: 9th tension (B3, 246.9 Hz +3 cents) - cluster bottom
    let osc4 = builder.add(OscillatorBlock::new(246.9, Waveform::Triangle, None));
    let gain4 = builder.add(GainBlock::new(-11.0, None));
    let pan4 = builder.add(PannerBlock::new(0.0));
    let lfo4 = builder.add(LfoBlock::new(0.041, 65.0, Waveform::Sine, None));

    builder.connect(osc4, 0, gain4, 0);
    builder.connect(gain4, 0, pan4, 0);
    builder.modulate(lfo4, pan4, "position");

    // Layer 5: m3 cluster (C4, 261.6 Hz) - semitone shimmer with B, filtered saw
    let osc5 = builder.add(OscillatorBlock::new(261.6, Waveform::Sawtooth, None));
    let lpf = builder.add(LowPassFilterBlock::new(900.0, 1.5));
    let filter_lfo = builder.add(LfoBlock::new(0.019, 400.0, Waveform::Sine, None));
    let gain5 = builder.add(GainBlock::new(-12.0, None));
    let pan5 = builder.add(PannerBlock::new(0.0));
    let pan_lfo5 = builder.add(LfoBlock::new(0.053, 75.0, Waveform::Sine, None));

    builder.connect(osc5, 0, lpf, 0);
    builder.modulate(filter_lfo, lpf, "cutoff");
    builder.connect(lpf, 0, gain5, 0);
    builder.connect(gain5, 0, pan5, 0);
    builder.modulate(pan_lfo5, pan5, "position");

    // Layer 6: Phrygian color (F4, 349.2 Hz -2 cents) - dark modal tension
    let osc6 = builder.add(OscillatorBlock::new(349.2, Waveform::Sine, None));
    let gain6 = builder.add(GainBlock::new(-14.0, None));
    let pan6 = builder.add(PannerBlock::new(0.0));
    let lfo6 = builder.add(LfoBlock::new(0.067, 85.0, Waveform::Sine, None));

    builder.connect(osc6, 0, gain6, 0);
    builder.connect(gain6, 0, pan6, 0);
    builder.modulate(lfo6, pan6, "position");

    builder.build()
}

fn main() {
    println!("Dark Am9 pad with Phrygian color");
    let player = Player::new(create_graph()).unwrap();
    let handle = player.play().unwrap();

    std::thread::sleep(Duration::from_secs(90));
    handle.stop();
}
