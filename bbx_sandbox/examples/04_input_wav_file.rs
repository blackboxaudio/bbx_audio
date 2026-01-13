//! WAV file input with LFO-modulated filter.
//!
//! Signal chain: FileInput -> LowPassFilter -> Gain -> Output
//! Modulation: LFO(0.25Hz) modulates filter cutoff

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
    let file_input = builder.add_file_input(Box::new(reader));
    let filter = builder.add_low_pass_filter(1000.0, 2.0);
    let gain = builder.add_gain(-3.0, None);
    let lfo = builder.add_lfo(0.25, 800.0, None);

    builder.connect(file_input, 0, filter, 0);
    builder.connect(filter, 0, gain, 0);
    builder.modulate(lfo, filter, "cutoff");

    builder.build()
}

fn main() {
    let player = Player::from_graph(create_graph());
    player.play(None);
}
