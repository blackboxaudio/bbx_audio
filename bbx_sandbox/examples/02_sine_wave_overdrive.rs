use bbx_dsp::{
    constants::DEFAULT_CONTEXT, effector::Effector, generator::Generator, generators::wave_table::Waveform,
    graph::Graph,
};
use bbx_sandbox::{player::Player, signal::Signal};

fn main() {
    // Create a `Graph` with the default context
    let mut graph = Graph::new(DEFAULT_CONTEXT);

    // Add nodes for an oscillator and overdrive
    let oscillator = graph.add_generator(Generator::WaveTable {
        frequency: 110.0,
        waveform: Waveform::Sine,
    });
    let overdrive = graph.add_effector(Effector::Overdrive());

    // Form the connection from the oscillator to the overdrive
    graph.create_connection(oscillator, overdrive);

    // Prepare the graph for playback
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
