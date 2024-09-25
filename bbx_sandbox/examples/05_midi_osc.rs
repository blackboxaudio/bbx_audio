use bbx_dsp::{generator::Generator, graph::Graph};
use bbx_midi::stream::MidiInputStream;
use bbx_sandbox::{
    constants::{DEFAULT_CONTEXT, SAMPLE_RATE},
    player::Player,
    signal::Signal,
};

pub fn create_graph() -> Graph {
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    graph.add_generator(Generator::WaveTable {
        sample_rate: SAMPLE_RATE,
        frequency: 110.0,
    });
    graph.prepare_for_playback();
    graph
}

fn main() {
    let stream = MidiInputStream::new(vec![], |message| {
        println!("{:#}", message);
    });
    let handle = stream.init();

    let signal = Signal::new(create_graph());
    Player::new(signal).play(Some(10));

    handle.join().unwrap();
}
