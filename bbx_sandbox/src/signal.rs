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
        // If the index has wrapped around back to the beginning,
        // then the graph needs to be processed again.
        if self.next_idx == 0 {
            // TODO: Remove this clone statement, and read value from graph
            // so the buffer can stay there.
            self.output = self.graph.evaluate().clone();
        }

        let prev_idx = self.next_idx;
        self.next_idx += 1;
        self.next_idx %= self.graph.context.buffer_size;
        self.output.first().unwrap()[prev_idx]
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
