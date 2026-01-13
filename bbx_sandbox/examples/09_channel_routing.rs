//! Channel routing demonstration.
//!
//! Demonstrates the ChannelRouterBlock with different routing modes:
//! Stereo, Left, Right, Swap, and mono summing.
//!
//! Creates two oscillators panned left and right, then routes through
//! a channel router that swaps the channels.
//!
//! Signal chain:
//!   Oscillator(440Hz) -> Panner(L) -+
//!                                   +-> ChannelRouter(Swap) -> Output
//!   Oscillator(660Hz) -> Panner(R) -+

use bbx_dsp::{
    blocks::{ChannelMode, ChannelRouterBlock, GainBlock, OscillatorBlock, PannerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Two oscillators at different frequencies
    let osc_left = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let osc_right = builder.add(OscillatorBlock::new(660.0, Waveform::Sine, None));

    // Pan them hard left and hard right
    let pan_left = builder.add(PannerBlock::new(-100.0));
    let pan_right = builder.add(PannerBlock::new(100.0));

    // Channel router to swap channels (so 440Hz ends up in right, 660Hz in left)
    let router = builder.add(ChannelRouterBlock::new(ChannelMode::Swap, false, false, false));

    // Master gain
    let gain = builder.add(GainBlock::new(-6.0, None));

    // Connect oscillators to panners
    builder.connect(osc_left, 0, pan_left, 0);
    builder.connect(osc_right, 0, pan_right, 0);

    // Merge panned signals into router
    builder.connect(pan_left, 0, router, 0);
    builder.connect(pan_left, 1, router, 1);
    builder.connect(pan_right, 0, router, 0);
    builder.connect(pan_right, 1, router, 1);

    // Router to gain
    builder.connect(router, 0, gain, 0);
    builder.connect(router, 1, gain, 1);

    builder.build()
}

fn main() {
    println!("Channel Routing Demo");
    println!("440Hz was panned left, 660Hz was panned right");
    println!("After channel swap: 440Hz is now in right ear, 660Hz is now in left ear");
    let player = Player::from_graph(create_graph());
    player.play(Some(10));
}
