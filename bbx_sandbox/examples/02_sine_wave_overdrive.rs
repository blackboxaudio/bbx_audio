use bbx_dsp::{constants::DEFAULT_CONTEXT, effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{player::Player, signal::Signal};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(DEFAULT_CONTEXT);

    let oscillator = graph.add_generator(Generator::WaveTable {
        sample_rate: DEFAULT_CONTEXT.sample_rate,
        frequency: 110.0,
    });
    let overdrive = graph.add_effector(Effector::Overdrive());
    graph.create_connection(oscillator, overdrive);
    graph.prepare_for_playback();

    graph
}

fn main() {
    let signal = Signal::new(create_graph());
    Player::new(signal).play(None);
}
