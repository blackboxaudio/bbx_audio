use rodio::{OutputStream, Source};
use bbx_dsp::graph::Graph;

const SAMPLE_RATE: usize = 44100;
// const OSC_FREQUENCY: f32 = 110.0;
const PLAYTIME_DURATION: u64 = 3;

pub fn run() {
    let signal = Graph::new(SAMPLE_RATE);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _result = stream_handle.play_raw(signal.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(PLAYTIME_DURATION));
}
