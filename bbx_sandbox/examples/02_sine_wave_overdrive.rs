use bbx_dsp::{block::Block, effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let mut oscillator = Block::new(Generator::new(SAMPLE_RATE, Some(110.0)).to_operation());
    let mut overdrive = Block::new(Effector::new().to_operation());

    graph.create_connection(&mut oscillator, &mut overdrive);

    graph.add_block(oscillator);
    graph.add_block(overdrive);

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
