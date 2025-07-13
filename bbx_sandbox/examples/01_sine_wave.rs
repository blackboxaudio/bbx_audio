use bbx_dsp::{
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::{player::Player, signal::Signal};

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(44100.0, 512, 2);

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine);

    let output = builder.add_output(2);

    builder.connect(oscillator, 0, output, 0);
    builder.connect(oscillator, 0, output, 1);

    builder.build()
}

fn main() {
    let graph = create_graph();
    let signal = Signal::new(graph);
    let player = Player::new(signal);
    player.play(Some(2));
}
