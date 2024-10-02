use bbx_dsp::{
    constants::DEFAULT_CONTEXT, effector::Effector, generator::Generator, generators::wave_table::Waveform,
    graph::Graph,
};
use bbx_sandbox::{player::Player, signal::Signal};

const NUM_OSCILLATORS: usize = 12;

const BASE_FREQUENCY: f32 = 55.0;

fn main() {
    // Create a `Graph` with the default context
    let mut graph = Graph::new(DEFAULT_CONTEXT);

    // Add a mixer node to sum the oscillators
    let mixer = graph.add_effector(Effector::Mixer);

    // Create a number of oscillator nodes and connect to the mixer
    for n in 0..NUM_OSCILLATORS {
        let oscillator = graph.add_generator(Generator::WaveTable {
            frequency: BASE_FREQUENCY * (n + 1) as f32,
            waveform: Waveform::Sine,
        });
        graph.create_connection(oscillator, mixer);
    }

    // Add an overdrive just because
    let overdrive = graph.add_effector(Effector::Overdrive);
    graph.create_connection(mixer, overdrive);

    // Prepare the graph for playback
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
