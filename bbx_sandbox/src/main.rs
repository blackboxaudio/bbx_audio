use rodio::{OutputStream, Source};

use crate::graphs::{GraphType, get_graph_from_type};
use crate::signal::Signal;

mod graphs;
mod signal;

const SAMPLE_RATE: usize = 44100;
// const OSC_FREQUENCY: f32 = 110.0;
const PLAYTIME_DURATION: u64 = 3;

fn main() {
    let graph = get_graph_from_type(SAMPLE_RATE, GraphType::SimpleOsc);
    let signal = Signal::new(SAMPLE_RATE, graph);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _result = stream_handle.play_raw(signal.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(PLAYTIME_DURATION));
}
