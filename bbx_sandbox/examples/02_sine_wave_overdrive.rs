use bbx_dsp::{block::Block, effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let mut oscillator_01 = Block::new(Generator::new(SAMPLE_RATE, Some(110.0)).to_operation());
    println!("OSC 1: {:#}", oscillator_01.id);
    let mut oscillator_02 = Block::new(Generator::new(SAMPLE_RATE, Some(220.0)).to_operation());
    println!("OSC 2: {:#}", oscillator_02.id);
    let mut oscillator_03 = Block::new(Generator::new(SAMPLE_RATE, Some(440.0)).to_operation());
    println!("OSC 3: {:#}", oscillator_03.id);

    let mut overdrive_01 = Block::new(Effector::new().to_operation());
    println!("OD 1: {:#}", overdrive_01.id);
    let mut overdrive_02 = Block::new(Effector::new().to_operation());
    println!("OD 2: {:#}", overdrive_02.id);
    let mut overdrive_03 = Block::new(Effector::new().to_operation());
    println!("OD 3: {:#}", overdrive_03.id);
    let mut overdrive_04 = Block::new(Effector::new().to_operation());
    println!("OD 4: {:#}", overdrive_04.id);

    oscillator_01.add_output(overdrive_01.id);
    oscillator_02.add_output(overdrive_02.id);
    oscillator_03.add_output(overdrive_03.id);
    overdrive_01.add_output(overdrive_02.id);
    overdrive_02.add_output(overdrive_03.id);
    overdrive_03.add_output(overdrive_04.id);

    graph.add_block(oscillator_01);
    graph.add_block(oscillator_02);
    graph.add_block(oscillator_03);

    graph.add_block(overdrive_01);
    graph.add_block(overdrive_02);
    graph.add_block(overdrive_03);
    graph.add_block(overdrive_04);

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
