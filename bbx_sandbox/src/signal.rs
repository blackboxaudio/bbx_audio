use std::time::Duration;

use bbx_dsp::{
    buffer::{AudioBuffer, Buffer},
    graph::Graph,
};
use rodio::Source;

pub struct Signal {
    graph: Graph,
    output: Vec<AudioBuffer<f32>>,
    next_idx: usize,
}

impl Signal {
    pub fn new(graph: Graph) -> Signal {
        let context = graph.context;
        Signal {
            graph,
            output: vec![AudioBuffer::new(context.buffer_size); context.num_channels],
            next_idx: 0,
        }
    }
}

impl Signal {
    fn process(&mut self) -> f32 {
        if self.next_idx == 0 {
            self.output = self.graph.evaluate().clone();
        }

        let sample = self.output.get(0).unwrap()[self.next_idx];
        self.next_idx += 1;
        self.next_idx %= self.graph.context.buffer_size;
        sample
    }
}

impl Iterator for Signal {
    type Item = f32;

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
        self.graph.sample_rate() as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
