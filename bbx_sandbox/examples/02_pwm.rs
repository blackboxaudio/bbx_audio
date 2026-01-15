use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bbx_dsp::{
    blocks::{LfoBlock, OscillatorBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_player::Player;
use rand::prelude::*;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mut rng = thread_rng();

    let oscillator = builder.add(OscillatorBlock::new(440.0, Waveform::Sawtooth, Some(rng.next_u64())));

    let lfo1 = builder.add(LfoBlock::new(1.0, 5.0, Waveform::Sine, Some(rng.next_u64())));
    let lfo2 = builder.add(LfoBlock::new(1.0, 2.0, Waveform::Sine, Some(rng.next_u64())));
    let lfo3 = builder.add(LfoBlock::new(1.0, 3.0, Waveform::Sine, Some(rng.next_u64())));
    builder.modulate(lfo1, oscillator, "Frequency");
    builder.modulate(lfo2, lfo1, "Depth");
    builder.modulate(lfo3, lfo2, "Frequency");

    builder.build()
}

fn main() {
    println!("PWM Modulation Demo - Sawtooth with cascading LFO modulation");
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
