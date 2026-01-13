//! Ambisonic panning drone example with binaural decoding.
//!
//! Creates a hypnotic ambient drone using quartal harmony (stacked 4ths)
//! spatially positioned in a 3D FOA (First Order Ambisonics) soundfield.
//! Each source orbits with independent LFO modulation of azimuth and elevation,
//! creating complex Lissajous-like spatial patterns. The ambisonic signal is
//! decoded binaurally for headphone listening.

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

    // Create mixer to sum all ambisonic sources (4 sources × 4 FOA channels = 16 inputs)
    let mixer_id = builder.add_mixer(NUM_SOURCES, num_ambi_channels);

    // Create binaural decoder for headphone output
    let decoder = builder.add_binaural_decoder(AMBISONIC_ORDER);

    // Connect mixer outputs to decoder inputs
    for ch in 0..num_ambi_channels {
        builder.connect(mixer_id, ch, decoder, ch);
    }

    // Open quartal voicing with P5 bass: A2 → E3 → D4 → G4

    // Layer 1: Root (A2, 110 Hz) - foundation, gentle drift
    let osc1 = builder.add_oscillator(110.0, Waveform::Sine, None);
    let gain1 = builder.add_gain(-6.0, None);
    let enc1 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo1_az = builder.add_lfo(0.015, 60.0, None);
    let lfo1_el = builder.add_lfo(0.012, 15.0, None);

    builder.connect(osc1, 0, gain1, 0);
    builder.connect(gain1, 0, enc1, 0);
    builder.modulate(lfo1_az, enc1, "azimuth");
    builder.modulate(lfo1_el, enc1, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc1, ch, mixer_id, 0 * num_ambi_channels + ch);
    }

    // Layer 2: Open 5th (E3, 164.81 Hz) - slightly more motion
    let osc2 = builder.add_oscillator(164.81, Waveform::Sine, None);
    let gain2 = builder.add_gain(-9.0, None);
    let enc2 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo2_az = builder.add_lfo(0.025, 90.0, None);
    let lfo2_el = builder.add_lfo(0.02, 30.0, None);

    builder.connect(osc2, 0, gain2, 0);
    builder.connect(gain2, 0, enc2, 0);
    builder.modulate(lfo2_az, enc2, "azimuth");
    builder.modulate(lfo2_el, enc2, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc2, ch, mixer_id, 1 * num_ambi_channels + ch);
    }

    // Layer 3: Upper voice (D4, 293.66 Hz) - moderate orbit
    let osc3 = builder.add_oscillator(293.66, Waveform::Sine, None);
    let gain3 = builder.add_gain(-12.0, None);
    let enc3 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo3_az = builder.add_lfo(0.04, 140.0, None);
    let lfo3_el = builder.add_lfo(0.035, 50.0, None);

    builder.connect(osc3, 0, gain3, 0);
    builder.connect(gain3, 0, enc3, 0);
    builder.modulate(lfo3_az, enc3, "azimuth");
    builder.modulate(lfo3_el, enc3, "elevation");

    for ch in 0..num_ambi_channels {
        builder.connect(enc3, ch, mixer_id, 2 * num_ambi_channels + ch);
    }

    // Layer 4: Top voice (G4, 392.0 Hz) - fastest, widest orbit
    let osc4 = builder.add_oscillator(392.0, Waveform::Sawtooth, None);
    let lpf = builder.add_low_pass_filter(1200.0, 1.2);
    let gain4 = builder.add_gain(-15.0, None);
    let enc4 = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    let lfo4_az = builder.add_lfo(0.065, 180.0, None);
    let lfo4_el = builder.add_lfo(0.055, 75.0, None);

    builder.connect(osc4, 0, lpf, 0);
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
    println!("Open quartal ambisonic pad (A-E-D-G) - best experienced with headphones!");
    let player = Player::from_graph(create_graph());
    player.play(Some(90));
}
