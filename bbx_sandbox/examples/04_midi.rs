use bbx_dsp::generator::Generator;
use bbx_dsp::graph::Graph;
use bbx_sandbox::constants::SAMPLE_RATE;
use bbx_sandbox::player::Player;
use bbx_sandbox::signal::Signal;

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(SAMPLE_RATE);
    graph.add_generator(Generator::WaveTable {
        sample_rate: SAMPLE_RATE,
        frequency: 110.0,
    });
    graph.prepare_for_playback();
    graph
}

fn main() {
    let signal = Signal::new(SAMPLE_RATE, create_graph());
    Player::new(signal).play(None);
}
