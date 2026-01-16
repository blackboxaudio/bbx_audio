//! Overdrive effect with LFO-modulated drive and DC blocking.
//!
//! Signal chain: Oscillator(220Hz, Saw) -> Overdrive -> DcBlocker -> Gain -> Output
//! Modulation: LFO(0.5Hz) modulates overdrive drive parameter

use std::time::Duration;

use bbx_dsp::{
    blocks::{DcBlockerBlock, GainBlock, LfoBlock, OscillatorBlock, OverdriveBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add(OscillatorBlock::new(220.0, Waveform::Sawtooth, None));
    let overdrive = builder.add(OverdriveBlock::new(5.0, 0.8, 0.5, DEFAULT_SAMPLE_RATE));
    let dc_blocker = builder.add(DcBlockerBlock::new(true));
    let gain = builder.add(GainBlock::new(-6.0, None));
    let lfo = builder.add(LfoBlock::new(0.5, 3.0, Waveform::Sine, None));

    builder.connect(oscillator, 0, overdrive, 0);
    builder.connect(overdrive, 0, dc_blocker, 0);
    builder.connect(dc_blocker, 0, gain, 0);
    builder.modulate(lfo, overdrive, "drive");

    builder.build()
}

fn main() {
    println!("Overdrive Demo - LFO-modulated drive with DC blocking");

    let player = Player::new(create_graph()).unwrap();
    let handle = player.play().unwrap();

    std::thread::sleep(Duration::from_secs(30));
    handle.stop();
}
