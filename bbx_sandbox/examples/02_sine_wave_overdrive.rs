use bbx_dsp::{block::Block, effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

const NUM_OVERDRIVE_EFFECTORS: usize = 32;
pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let mut oscillator = Generator::new(graph.sample_rate());
    oscillator.set_frequency(220.0);
    graph.add_block(Block::Generator(oscillator));

    for _ in 0..NUM_OVERDRIVE_EFFECTORS {
        graph.add_block(Block::Effector(Effector::new()));
    }

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
