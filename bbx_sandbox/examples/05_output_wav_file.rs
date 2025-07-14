use bbx_dsp::{
    context::{DEFAULT_BUFFER_SIZE, DEFAULT_SAMPLE_RATE},
    graph::{Graph, GraphBuilder},
    waveform::Waveform,
};
use bbx_file::writers::wav::WavFileWriter;
use bbx_sandbox::{player::Player, signal::Signal};

fn create_graph() -> Graph<f32> {
    let mut builder = GraphBuilder::new(DEFAULT_SAMPLE_RATE, DEFAULT_BUFFER_SIZE, 2);

    let oscillator = builder.add_oscillator(440.0, Waveform::Sine);

    let lfo = builder.add_lfo(22.5, 100.0);
    builder.modulate(lfo, oscillator, "Frequency");

    let output = builder.add_output(2);

    builder.connect(oscillator, 0, output, 0);
    builder.connect(oscillator, 0, output, 1);

    let mut file_path = std::env::current_dir().unwrap().to_str().unwrap().to_owned();
    file_path.push_str("/bbx_sandbox/examples/05_output_wav_file.wav");

    let writer = WavFileWriter::new(file_path.as_str(), DEFAULT_SAMPLE_RATE, 2).unwrap();
    let file_output = builder.add_file_output(Box::new(writer));

    builder.connect(oscillator, 0, file_output, 0);
    builder.connect(oscillator, 0, file_output, 1);

    builder.build()
}

fn main() {
    let graph = create_graph();
    let signal = Signal::new(graph);
    let player = Player::new(signal);
    player.play(Some(2));
}
