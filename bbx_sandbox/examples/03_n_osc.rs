use bbx_dsp::{
    constants::DEFAULT_CONTEXT, effector::Effector, generator::Generator, generators::wave_table::Waveform,
    graph::Graph,
};
use bbx_sandbox::{player::Player, signal::Signal};

const NUM_OSCILLATORS: usize = 6;

const BASE_FREQUENCY: f32 = 55.0;
const MAX_FREQUENCY: f32 = 999.0;

fn main() {
    // Create a `Graph` with the default context
    let mut graph = Graph::new(DEFAULT_CONTEXT);

    // Add a mixer node to sum the oscillators
    let mixer = graph.add_effector(Effector::Mixer);

    // Create a number of oscillator nodes and connect to the mixer
    for n in 0..NUM_OSCILLATORS {
        let frequency = (n as f32 / NUM_OSCILLATORS as f32) * MAX_FREQUENCY + BASE_FREQUENCY;
        let oscillator = graph.add_generator(Generator::WaveTable {
            frequency,
            waveform: Waveform::Sine,
        });
        graph.create_connection(oscillator, mixer);
    }

    // Add an overdrive just because
    let overdrive = graph.add_effector(Effector::Overdrive);
    graph.create_connection(mixer, overdrive);

    // Add a filter because it sounds harsh
    let filter = graph.add_effector(Effector::Filter(DEFAULT_CONTEXT, MAX_FREQUENCY / 2.0, 1.5));
    graph.create_connection(overdrive, filter);

    // Prepare the graph for playback
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
