use std::env;

use bbx_dsp::{constants::DEFAULT_CONTEXT, generator::Generator, graph::Graph};
use bbx_sandbox::{player::Player, signal::Signal};

fn main() {
    // Construct string path to the example wav file
    let mut file_path = env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/examples/06_sample.wav");

    // Create a `Graph` with the default context, add a wave table generator,
    // and prepare it for playback
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    graph.add_generator(Generator::FileReader { file_path });
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
