use std::time::Duration;

use bbx_dsp::{graph::Graph, sample::Sample};
use rodio::{OutputStream, Source};

use crate::signal::Signal;

const DEFAULT_PLAYTIME_DURATION_SECONDS: usize = usize::MAX;

pub struct Player<S: Sample> {
    signal: Signal<S>,
}

impl<S: Sample> Player<S> {
    pub fn new(signal: Signal<S>) -> Self {
        Self { signal }
    }

    pub fn from_graph(graph: Graph<S>) -> Self {
        let signal = Signal::new(graph);
        Self { signal }
    }
}

impl Player<f32> {
    pub fn play(self, duration: Option<usize>) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let _result = stream_handle.play_raw(self.signal.convert_samples());

        std::thread::sleep(Duration::from_secs(
            duration.unwrap_or(DEFAULT_PLAYTIME_DURATION_SECONDS) as u64,
        ))
    }
}
