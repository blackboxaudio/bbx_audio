use std::time::Duration;

use rodio::Source;

use bbx_dsp::graph::Graph;
use bbx_dsp::sample::Sample;

pub struct Signal {
    sample_rate: usize,
    graph: Graph,
}

impl Signal {
    pub fn new (sample_rate: usize, graph: Graph) -> Signal {
        return Signal {
            sample_rate,
            graph,
        }
    }
}

impl Signal {
    fn process(&self) -> Sample<f32> {
        return self.graph.evaluate();
    }
}

impl Iterator for Signal {
    type Item = Sample<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.process());
    }
}

impl Source for Signal {
    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate as u32;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}
