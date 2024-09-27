use bbx_dsp::{constants::DEFAULT_CONTEXT, generator::Generator, generators::wave_table::Waveform, graph::Graph};
use bbx_sandbox::{player::Player, signal::Signal};

fn main() {
    // Create a `Graph` with the default context, add a wave table generator,
    // and prepare it for black
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    graph.add_generator(Generator::WaveTable {
        frequency: 110.0,
        waveform: Waveform::Sine,
    });
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
