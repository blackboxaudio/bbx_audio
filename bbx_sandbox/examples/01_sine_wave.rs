//! Basic oscillator with gain control example.
//!
//! Signal chain: Oscillator(440Hz, Sine) -> Gain(-6dB) -> Output

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::{
    blocks::{GainBlock, OscillatorBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(-6.0, None));

    builder.connect(oscillator, 0, gain, 0);

    builder.build()
}

fn main() {
    println!("Basic Sine Wave - 440Hz oscillator with -6dB gain");
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
