//! Offline file processing pipeline.
//!
//! Demonstrates reading a WAV file, processing it through effects,
//! and writing the result to a new WAV file.
//!
//! Signal chain: FileInput -> LowPassFilter -> Overdrive -> DcBlocker -> FileOutput

use bbx_dsp::{
    blocks::{DcBlockerBlock, FileInputBlock, FileOutputBlock, LowPassFilterBlock, OverdriveBlock},
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
};
use bbx_file::{readers::wav::WavFileReader, writers::wav::WavFileWriter};
use bbx_sandbox::player::Player;

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let mut input_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    input_path.push_str("/bbx_sandbox/examples/04_input_wav_file.wav");

    let mut output_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    output_path.push_str("/bbx_sandbox/examples/13_processed_output.wav");

    let reader = WavFileReader::from_path(input_path.as_str()).unwrap();
    let file_input = builder.add(FileInputBlock::new(Box::new(reader)));

    let filter = builder.add(LowPassFilterBlock::new(2000.0, 1.5));
    let overdrive = builder.add(OverdriveBlock::new(3.0, 0.8, 0.6, DEFAULT_SAMPLE_RATE));
    let dc_blocker = builder.add(DcBlockerBlock::new(true));

    let writer = WavFileWriter::new(output_path.as_str(), DEFAULT_SAMPLE_RATE, 2).unwrap();
    let file_output = builder.add(FileOutputBlock::new(Box::new(writer)));

    // Build signal chain
    builder.connect(file_input, 0, filter, 0);
    builder.connect(filter, 0, overdrive, 0);
    builder.connect(overdrive, 0, dc_blocker, 0);
    builder.connect(dc_blocker, 0, file_output, 0);

    builder.build()
}

fn main() {
    println!("File Processing Demo");
    println!("Processing: 04_input_wav_file.wav -> 13_processed_output.wav");
    println!("Effects: LowPassFilter -> Overdrive -> DC Blocker");

    let player = Player::from_graph(create_graph());
    player.play(Some(10));

    println!("Processing complete!");
}
