use bbx_dsp::{block::Block, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE,);

    let mut oscillator = Generator::new(graph.sample_rate(),);
    oscillator.set_frequency(220.0,);
    graph.add_block(Block::Generator(oscillator,),);

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph(),);
    Player::new(signal,).play();
}
