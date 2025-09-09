use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine, None);
    let overdrive = builder.add_overdrive(5.0, 1.0, 1.0, DEFAULT_SAMPLE_RATE);
    builder.connect(oscillator, 0, overdrive, 0);

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
