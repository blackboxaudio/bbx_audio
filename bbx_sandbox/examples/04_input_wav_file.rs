use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};
use bbx_file::readers::wav::WavFileReader;
use bbx_sandbox::{player::Player, signal::Signal};

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mut file_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/bbx_sandbox/examples/04_input_wav_file.wav");

    let reader = WavFileReader::from_path(file_path.as_str()).unwrap();
    let file_input = builder.add_file_input(Box::new(reader));

    let output = builder.add_output(2);

    builder.connect(file_input, 0, output, 0);
    builder.connect(file_input, 0, output, 1);

    builder.build()
}

fn main() {
    let graph = create_graph();
    let signal = Signal::new(graph);
    let player = Player::new(signal);
    player.play(Some(2));
}
