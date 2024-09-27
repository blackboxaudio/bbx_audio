use bbx_dsp::{constants::DEFAULT_CONTEXT, generator::Generator, generators::wave_table::Waveform, graph::Graph};
use bbx_midi::stream::MidiInputStream;
use bbx_sandbox::{player::Player, signal::Signal};

fn main() {
    // Create a new MIDI input stream with a callback for when
    // a MIDI message is received (requires specifying a MIDI input
    // via the console)
    let stream = MidiInputStream::new(vec![], |message| {
        println!("{:#}", message);
    });

    // Initialize the stream and listen for incoming MIDI events
    let handle = stream.init();

    // Create a `Graph` with the default context, add a wave table generator,
    // and prepare it for black
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    graph.add_generator(Generator::WaveTable {
        frequency: 110.0,
        waveform: Waveform::Sine,
    });
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(Some(10));

    // Wait for the user to cancel the program
    handle.join().unwrap();
}
