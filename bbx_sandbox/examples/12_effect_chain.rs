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

    // Overdrive for saturation
    let overdrive = builder.add_overdrive(4.0, 0.7, 0.5, DEFAULT_SAMPLE_RATE);

    // DC blocker after distortion
    let dc_blocker = builder.add_dc_blocker(true);

    // Resonant low-pass filter
    let filter = builder.add_low_pass_filter(1200.0, 3.0);

    // Stereo panner
    let panner = builder.add_panner_stereo(0.0);

    // Output gain
    let gain = builder.add_gain(-9.0, None);

    // LFOs for modulation
    let lfo_filter = builder.add_lfo(0.25, 800.0, None);
    let lfo_pan = builder.add_lfo(0.1, 60.0, None);

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
    let player = Player::from_graph(create_graph());
    player.play(None);
}
