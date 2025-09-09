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

    let oscillator = builder.add_oscillator(440.0, Waveform::Sawtooth, Some(rng.next_u64()));

    let lfo1 = builder.add_lfo(1.0, 5.0, Some(rng.next_u64()));
    let lfo2 = builder.add_lfo(1.0, 2.0, Some(rng.next_u64()));
    let lfo3 = builder.add_lfo(1.0, 3.0, Some(rng.next_u64()));
    builder.modulate(lfo1, oscillator, "Frequency");
    builder.modulate(lfo2, lfo1, "Depth");
    builder.modulate(lfo3, lfo2, "Frequency");

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
