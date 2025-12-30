use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};
use bbx_file::readers::wav::WavFileReader;
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mut file_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/bbx_sandbox/examples/04_input_wav_file.wav");

    let reader = WavFileReader::from_path(file_path.as_str()).unwrap();
    let _file_input = builder.add_file_input(Box::new(reader));

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
