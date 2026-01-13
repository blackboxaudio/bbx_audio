//! Polygonia-style spatial dark drone with ambisonic panning.
//!
//! Creates a hypnotic techno drone using a Dm9(no3) "Dorian suspension" voicing.
//! The missing 3rd creates harmonic ambiguity while the 9th adds warmth against
//! the dark foundation. Four sources orbit through 3D FOA space with asymmetric
//! LFO patterns, creating complex Lissajous-like spatial movement that never
//! quite repeats. Best experienced with headphones.

use bbx_dsp::{
    channel::ChannelLayout,
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

const AMBISONIC_ORDER: usize = 1;
const NUM_SOURCES: usize = 4;

fn create_graph() -> Graph<f32> {
    let num_ambi_channels = ChannelLayout::ambisonic_channel_count(AMBISONIC_ORDER);

    let mut builder = GraphBuilder::with_layout(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, ChannelLayout::Stereo);

    let mixer_id = builder.add_mixer(NUM_SOURCES, num_ambi_channels);
    let decoder = builder.add_binaural_decoder(AMBISONIC_ORDER);

    for ch in 0..num_ambi_channels {
        builder.connect(mixer_id, ch, decoder, ch);
    }

    // Layer 1: Root (D2, 73.4 Hz) - dark foundation, slow drift
    let osc1 = builder.add_oscillator(73.4, Waveform::Sine, None);
    let gain1 = builder.add_gain(-6.0, None);
    let enc1 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo1_az = builder.add_lfo(0.013, 50.0, None);
    let lfo1_el = builder.add_lfo(0.011, 12.0, None);

    builder.connect(osc1, 0, gain1, 0);
    builder.connect(gain1, 0, enc1, 0);
    builder.modulate(lfo1_az, enc1, "azimuth");
    builder.modulate(lfo1_el, enc1, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc1, ch, mixer_id, 0 * num_ambi_channels + ch);
    }

    // Layer 2: P5 (A2, 110.0 Hz) - anchor, gentle orbit
    let osc2 = builder.add_oscillator(110.0, Waveform::Sine, None);
    let gain2 = builder.add_gain(-8.0, None);
    let enc2 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo2_az = builder.add_lfo(0.019, 80.0, None);
    let lfo2_el = builder.add_lfo(0.017, 25.0, None);

    builder.connect(osc2, 0, gain2, 0);
    builder.connect(gain2, 0, enc2, 0);
    builder.modulate(lfo2_az, enc2, "azimuth");
    builder.modulate(lfo2_el, enc2, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc2, ch, mixer_id, 1 * num_ambi_channels + ch);
    }

    // Layer 3: m7 (C4, 261.6 Hz +2 cents) - minor color, moderate motion
    let osc3 = builder.add_oscillator(261.9, Waveform::Triangle, None);
    let gain3 = builder.add_gain(-11.0, None);
    let enc3 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo3_az = builder.add_lfo(0.029, 120.0, None);
    let lfo3_el = builder.add_lfo(0.023, 40.0, None);

    builder.connect(osc3, 0, gain3, 0);
    builder.connect(gain3, 0, enc3, 0);
    builder.modulate(lfo3_az, enc3, "azimuth");
    builder.modulate(lfo3_el, enc3, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc3, ch, mixer_id, 2 * num_ambi_channels + ch);
    }

    // Layer 4: 9th (E4, 329.6 Hz) - tension/warmth, fastest orbit with filter
    let osc4 = builder.add_oscillator(329.6, Waveform::Sawtooth, None);
    let lpf = builder.add_low_pass_filter(800.0, 1.8);
    let filter_lfo = builder.add_lfo(0.021, 350.0, None);
    let gain4 = builder.add_gain(-14.0, None);
    let enc4 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo4_az = builder.add_lfo(0.043, 160.0, None);
    let lfo4_el = builder.add_lfo(0.037, 60.0, None);

    builder.connect(osc4, 0, lpf, 0);
    builder.modulate(filter_lfo, lpf, "cutoff");
    builder.connect(lpf, 0, gain4, 0);
    builder.connect(gain4, 0, enc4, 0);
    builder.modulate(lfo4_az, enc4, "azimuth");
    builder.modulate(lfo4_el, enc4, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc4, ch, mixer_id, 3 * num_ambi_channels + ch);
    }

    builder.build()
}

fn main() {
    println!("Dm9(no3) spatial drone - Polygonia style - best with headphones!");
    let player = Player::from_graph(create_graph());
    player.play(Some(90));
}
