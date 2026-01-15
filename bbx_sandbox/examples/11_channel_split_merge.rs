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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bbx_dsp::{
    blocks::{
        ChannelMergerBlock, ChannelSplitterBlock, GainBlock, LfoBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock,
    },
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Rich harmonic source
    let oscillator = builder.add(OscillatorBlock::new(110.0, Waveform::Sawtooth, None));

    // Pan to create stereo signal with LFO
    let panner = builder.add(PannerBlock::new(0.0));
    let pan_lfo = builder.add(LfoBlock::new(0.2, 80.0, Waveform::Sine, None));

    // Split stereo into separate channels
    let splitter = builder.add(ChannelSplitterBlock::new(2));

    // Different filters for each channel
    let filter_left = builder.add(LowPassFilterBlock::new(500.0, 2.0));
    let filter_right = builder.add(LowPassFilterBlock::new(2000.0, 2.0));

    // Merge back to stereo
    let merger = builder.add(ChannelMergerBlock::new(2));

    // Output gain
    let gain = builder.add(GainBlock::new(-6.0, None));

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
