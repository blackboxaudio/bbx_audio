use std::time::Duration;

use bbx_buffer::buffer::{AudioBuffer, Buffer};
use bbx_dsp::graph::Graph;
use rodio::Source;

pub struct Signal {
    graph: Graph,
    output: Vec<AudioBuffer<f32>>,
    channel_idx: usize,
    sample_idx: usize,
}

impl Signal {
    pub fn new(graph: Graph) -> Signal {
        let context = graph.context;
        Signal {
            graph,
            output: vec![AudioBuffer::new(context.buffer_size); context.num_channels],
            channel_idx: 0,
            sample_idx: 0,
        }
    }
}

impl Signal {
    fn process(&mut self) -> f32 {
        // If both indices have wrapped around back to the beginning,
        // then the graph needs to be processed again.
        if self.channel_idx == 0 && self.sample_idx == 0 {
            let result_output = self.graph.evaluate();
            for (channel_idx, channel_buffer) in self.output.iter_mut().enumerate() {
                for sample_idx in 0..channel_buffer.len() {
                    channel_buffer[sample_idx] = result_output[channel_idx][sample_idx];
                }
            }
        }

        let sample = self.output[self.channel_idx][self.sample_idx];

        // `rodio` is expecting interleaved samples, meaning a format
        // where there is only one buffer with the samples for each channel
        // being placed consecutively. Because of that, we have to increment the
        // channel index every time and only increment the sample index when the
        // channel index has to be wrapped around.
        //
        // Non-Interleaved: [L1, L2, L3, R1, R2, R3]
        // Interleaved:     [L1, R1, L2, R2, L3, R3]
        self.channel_idx += 1;
        if self.channel_idx >= self.graph.context.num_channels {
            self.channel_idx = 0;
            self.sample_idx += 1;
            self.sample_idx %= self.graph.context.buffer_size;
        }

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
        self.graph.context.num_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.graph.context.sample_rate as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
