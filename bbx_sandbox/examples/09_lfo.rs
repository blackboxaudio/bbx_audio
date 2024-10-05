use bbx_dsp::{
    constants::DEFAULT_CONTEXT,
    effector::Effector,
    generator::Generator,
    generators::wave_table::Waveform,
    graph::Graph,
    modulator::{ModulationDestination, Modulator},
};
use bbx_sandbox::{player::Player, signal::Signal};

fn main() {
    // Create a `Graph` with the default context, add a wave table generator,
    // and prepare it for playback
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    let lfo1 = graph.add_modulator(Modulator::LowFrequencyOscillator { frequency: 330.0 });
    let lfo2 = graph.add_modulator(Modulator::LowFrequencyOscillator { frequency: 660.0 });
    let osc = graph.add_generator(Generator::WaveTable {
        frequency: 110.0,
        waveform: Waveform::Sine,
    });
    let filter = graph.add_effector(Effector::Filter(DEFAULT_CONTEXT, 1000.0, 1.6));
    graph.create_modulation(lfo1, osc, ModulationDestination::Frequency);
    graph.create_modulation(lfo2, lfo1, ModulationDestination::Frequency);
    graph.create_connection(osc, filter);
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
