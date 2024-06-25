mod graphs;
mod signal;

use rodio::{OutputStream, Source};

use crate::signal::Signal;

const SAMPLE_RATE: usize = 44100;
// const OSC_FREQUENCY: f32 = 110.0;
const PLAYTIME_DURATION: u64 = 3;

fn main() {
    let signal = Signal::new(SAMPLE_RATE);

    // Create DSP Graph and attach to Signal
    let _graph = graphs::simple_synth::create_simple_synth();
    // signal.attach_graph(_graph);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _result = stream_handle.play_raw(signal.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(PLAYTIME_DURATION));
}
