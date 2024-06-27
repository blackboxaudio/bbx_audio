use bbx_dsp::{generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);
    graph.add_generator(Generator::new(SAMPLE_RATE, Some(110.0)));
    graph.prepare_for_playback();
    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
