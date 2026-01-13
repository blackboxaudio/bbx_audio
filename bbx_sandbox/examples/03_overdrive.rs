//! Overdrive effect with LFO-modulated drive and DC blocking.
//!
//! Signal chain: Oscillator(220Hz, Saw) -> Overdrive -> DcBlocker -> Gain -> Output
//! Modulation: LFO(0.5Hz) modulates overdrive drive parameter

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add_oscillator(220.0, Waveform::Sawtooth, None);
    let overdrive = builder.add_overdrive(5.0, 0.8, 0.5, DEFAULT_SAMPLE_RATE);
    let dc_blocker = builder.add_dc_blocker(true);
    let gain = builder.add_gain(-6.0, None);
    let lfo = builder.add_lfo(0.5, 3.0, None);

    builder.connect(oscillator, 0, overdrive, 0);
    builder.connect(overdrive, 0, dc_blocker, 0);
    builder.connect(dc_blocker, 0, gain, 0);
    builder.modulate(lfo, overdrive, "drive");

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
