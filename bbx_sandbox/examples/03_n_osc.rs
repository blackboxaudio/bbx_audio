use bbx_dsp::{effector::Effector, generator::Generator, graph::Graph};
use bbx_sandbox::{constants::SAMPLE_RATE, player::Player, signal::Signal};

const NUM_OSCILLATORS: usize = 8;

const BASE_FREQUENCY: f32 = 49.0;

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);

    let mixer = graph.add_effector(Effector::Mixer());
    for n in 0..NUM_OSCILLATORS {
        let oscillator = graph.add_generator(Generator::Wavetable {
            sample_rate: SAMPLE_RATE,
            frequency: BASE_FREQUENCY * (n + 1) as f32,
        });
        graph.create_connection(oscillator, mixer)
    }

    let overdrive = graph.add_effector(Effector::Overdrive());
    graph.create_connection(mixer, overdrive);

    graph.prepare_for_playback();

    return graph;
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play();
}
