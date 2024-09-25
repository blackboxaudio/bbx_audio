use std::time::Duration;
use rodio::Source;
use bbx_dsp::{graph::Graph, sample::Sample};

pub struct Signal {
    sample_rate: usize,
    graph: Graph,
}

impl Signal {
    pub fn new(sample_rate: usize, graph: Graph) -> Signal {
        Signal { sample_rate, graph }
    }
}

impl Signal {
    fn process(&mut self) -> Sample {
        self.graph.evaluate()
    }
}

impl Iterator for Signal {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for Signal {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
