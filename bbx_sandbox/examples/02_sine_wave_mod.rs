use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;
use rand::prelude::*;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mut rng = thread_rng();

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine, Some(rng.next_u64()));

    let lfo = builder.add_lfo(22.5, 100.0, Some(rng.next_u64()));
    builder.modulate(lfo, oscillator, "Frequency");

    let output = builder.add_output(2);

    builder.connect(oscillator, 0, output, 0);
    builder.connect(oscillator, 0, output, 1);

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
