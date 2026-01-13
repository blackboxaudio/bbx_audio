//! Basic oscillator with gain control example.
//!
//! Signal chain: Oscillator(440Hz, Sine) -> Gain(-6dB) -> Output

use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine, None);
    let gain = builder.add_gain(-6.0, None);

    builder.connect(oscillator, 0, gain, 0);

    builder.build()
}

fn main() {
    println!("Basic Sine Wave - 440Hz oscillator with -6dB gain");
    let player = Player::from_graph(create_graph());
    player.play(None);
}
