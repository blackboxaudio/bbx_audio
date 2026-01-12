//! Ambisonic audio processing example.
//!
//! Encodes a mono source to first-order ambisonics, then decodes to stereo.
//! An LFO rotates the source around the listener.

use bbx_dsp::{
    channel::ChannelLayout,
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

const AMBISONIC_ORDER: usize = 1;

fn create_graph() -> Graph<f32> {
    // Create graph with stereo output (decoded from ambisonics)
    let mut builder = GraphBuilder::with_layout(
        DEFAULT_SAMPLE_RATE,
        DEFAULT_BUFFER_SIZE,
        ChannelLayout::Stereo,
    );

    // Mono oscillator source
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

    // Encode to first-order ambisonics
    let encoder = builder.add_panner_ambisonic(AMBISONIC_ORDER);
    builder.connect(osc, 0, encoder, 0);

    // LFO to rotate the source around the listener
    // Depth of 180 gives azimuth range of -180 to +180 degrees (full circle)
    let lfo = builder.add_lfo(0.25, 180.0, None);
    builder.modulate(lfo, encoder, "azimuth");

    // Decode back to stereo for playback
    let decoder = builder.add_ambisonic_decoder(AMBISONIC_ORDER, ChannelLayout::Stereo);
    for ch in 0..ChannelLayout::ambisonic_channel_count(AMBISONIC_ORDER) {
        builder.connect(encoder, ch, decoder, ch);
    }

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(Some(10));
}
