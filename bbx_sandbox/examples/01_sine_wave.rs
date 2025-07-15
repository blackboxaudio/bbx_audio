use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let _oscillator = builder.add_oscillator(440.0, Waveform::Sine, None);

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
