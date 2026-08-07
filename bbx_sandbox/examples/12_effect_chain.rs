//! Multi-stage effect chain with modulation.
//!
//! Demonstrates a complete realistic signal chain with:
//! - Overdrive for harmonic saturation
//! - DC blocking after distortion
//! - LFO-modulated filter
//! - LFO-modulated stereo panning
//! - Proper gain staging
//!
//! Signal chain:
//!   Oscillator(110Hz, Saw) -> Overdrive -> DcBlocker -> LowPassFilter -> Panner -> Gain -> Output
//! Modulation:
//!   LFO1(0.25Hz) -> Filter cutoff
//!   LFO2(0.1Hz)  -> Panner position

use std::time::Duration;

use bbx_dsp::{
    blocks::{DcBlockerBlock, GainBlock, LfoBlock, LowPassFilterBlock, OscillatorBlock, OverdriveBlock, PannerBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    // Rich harmonic source
    let oscillator = builder.add(OscillatorBlock::new(110.0, Waveform::Sawtooth, None));

    // Overdrive for saturation
    let overdrive = builder.add(OverdriveBlock::new(4.0, 0.7, 0.5, DEFAULT_SAMPLE_RATE));

    // DC blocker after distortion
    let dc_blocker = builder.add(DcBlockerBlock::new(true));

    // Resonant low-pass filter
    let filter = builder.add(LowPassFilterBlock::new(1200.0, 3.0));

    // Stereo panner
    let panner = builder.add(PannerBlock::new(0.0));

    // Output gain
    let gain = builder.add(GainBlock::new(-9.0, None));

    // LFOs for modulation
    let lfo_filter = builder.add(LfoBlock::new(0.25, 800.0, Waveform::Sine, None));
    let lfo_pan = builder.add(LfoBlock::new(0.1, 60.0, Waveform::Sine, None));

    // Build audio signal chain
    builder.connect(oscillator, 0, overdrive, 0);
    builder.connect(overdrive, 0, dc_blocker, 0);
    builder.connect(dc_blocker, 0, filter, 0);
    builder.connect(filter, 0, panner, 0);
    builder.connect(panner, 0, gain, 0);
    builder.connect(panner, 1, gain, 1);

    // Apply modulation
    builder.modulate(lfo_filter, filter, "cutoff");
    builder.modulate(lfo_pan, panner, "position");

    builder.build()
}

fn main() {
    println!("Multi-Stage Effect Chain Demo");
    println!("Overdrive -> DC Blocker -> Modulated Filter -> Modulated Panner");

    let player = Player::new(create_graph()).unwrap();
    let handle = player.play().unwrap();

    std::thread::sleep(Duration::from_secs(30));
    handle.stop();
}
