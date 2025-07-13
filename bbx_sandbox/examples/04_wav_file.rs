// use std::env;
use bbx_dsp::{
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_sandbox::{player::Player, signal::Signal};

// fn main() {
//     // Construct string path to the example wav file
//     let mut file_path = env::current_dir().unwrap().to_str().unwrap().to_owned();
//     file_path.push_str("/examples/04_wav_file.wav");
//
//     // Create a `Graph` with the default context, add a wave table generator,
//     // and prepare it for playback
//     let mut graph = Graph::new(DEFAULT_CONTEXT);
//     graph.add_generator(Generator::FileReader { file_path });
//     graph.prepare_for_playback();
//
//     // Play a `Signal` created from the graph
//     let signal = Signal::new(graph);
//     Player::new(signal).play(None);
// }

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(44100.0, 512, 2);

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine);

    let output = builder.add_output(2);

    builder.connect(oscillator, 0, output, 0);
    builder.connect(oscillator, 0, output, 1);

    builder.build()
}

fn main() {
    let graph = create_graph();
    let signal = Signal::new(graph);
    let player = Player::new(signal);
    player.play(Some(2));
}
