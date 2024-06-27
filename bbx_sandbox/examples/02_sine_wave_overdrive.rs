use bbx_dsp::{block::Block, effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let mut oscillator_01 = Block::new(Generator::new(SAMPLE_RATE, Some(110.0)).to_operation());
    println!("OSC 1: {:#}", oscillator_01.id);
    let mut oscillator_02 = Block::new(Generator::new(SAMPLE_RATE, Some(220.0)).to_operation());
    println!("OSC 2: {:#}", oscillator_02.id);

    let mut overdrive = Block::new(Effector::new().to_operation());
    println!("OD 1: {:#}", overdrive.id);

    graph.create_connection(&mut oscillator_01, &mut overdrive);
    graph.create_connection(&mut oscillator_02, &mut overdrive);

    graph.add_block(oscillator_01);
    graph.add_block(oscillator_02);
    graph.add_block(overdrive);

    graph.prepare_for_playback();

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
