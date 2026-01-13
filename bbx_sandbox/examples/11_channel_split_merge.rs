//! Channel splitting and merging for parallel processing.
//!
//! Demonstrates using ChannelSplitterBlock and ChannelMergerBlock to
//! process channels independently with different effects, then recombine.
//!
//! Signal chain:
//!   Oscillator(stereo via panner)
//!     -> Splitter
//!       -> Left:  LowPassFilter(500Hz)  -+
//!       -> Right: LowPassFilter(2000Hz) -+-> Merger -> Gain -> Output

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Rich harmonic source
    let oscillator = builder.add_oscillator(110.0, Waveform::Sawtooth, None);

    // Pan to create stereo signal with LFO
    let panner = builder.add_panner_stereo(0.0);
    let pan_lfo = builder.add_lfo(0.2, 80.0, None);

    // Split stereo into separate channels
    let splitter = builder.add_channel_splitter(2);

    // Different filters for each channel
    let filter_left = builder.add_low_pass_filter(500.0, 2.0);
    let filter_right = builder.add_low_pass_filter(2000.0, 2.0);

    // Merge back to stereo
    let merger = builder.add_channel_merger(2);

    // Output gain
    let gain = builder.add_gain(-6.0, None);

    // Build signal chain
    builder.connect(oscillator, 0, panner, 0);
    builder.modulate(pan_lfo, panner, "position");

    // Panner stereo output to splitter
    builder.connect(panner, 0, splitter, 0);
    builder.connect(panner, 1, splitter, 1);

    // Splitter outputs to different filters
    builder.connect(splitter, 0, filter_left, 0);
    builder.connect(splitter, 1, filter_right, 0);

    // Filters to merger
    builder.connect(filter_left, 0, merger, 0);
    builder.connect(filter_right, 0, merger, 1);

    // Merger to gain
    builder.connect(merger, 0, gain, 0);
    builder.connect(merger, 1, gain, 1);

    builder.build()
}

fn main() {
    println!("Channel Split/Merge Demo");
    println!("Left channel: 500Hz filter (darker)");
    println!("Right channel: 2000Hz filter (brighter)");
    let player = Player::from_graph(create_graph());
    player.play(None);
}
