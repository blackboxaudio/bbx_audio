use bbx_dsp::{effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let oscillator = graph.add_generator(Generator::Wavetable {
        sample_rate: SAMPLE_RATE,
        frequency: 110.0,
    });
    let overdrive = graph.add_effector(Effector::Overdrive());
    graph.create_connection(oscillator, overdrive);

    graph.prepare_for_playback();

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
