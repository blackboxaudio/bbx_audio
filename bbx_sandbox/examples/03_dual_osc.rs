use bbx_dsp::{effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let oscillator1 = graph.add_generator(Generator::new(SAMPLE_RATE, Some(60.0)));
    let oscillator2 = graph.add_generator(Generator::new(SAMPLE_RATE, Some(120.0)));
    let oscillator3 = graph.add_generator(Generator::new(SAMPLE_RATE, Some(180.0)));
    let mixer = graph.add_effector(Effector::Mixer());
    let overdrive = graph.add_effector(Effector::Overdrive());

    graph.create_connection(oscillator1, mixer);
    graph.create_connection(oscillator2, mixer);
    // graph.create_connection(oscillator3, mixer);
    graph.create_connection(mixer, overdrive);

    graph.prepare_for_playback();

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
