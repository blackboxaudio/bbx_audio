use std::time::Duration;

use bbx_dsp::{
    buffer::{AudioBuffer, Buffer},
    graph::Graph,
    sample::Sample,
};
use rodio::Source;

pub struct Signal<S: Sample> {
    graph: Graph<S>,
    output_buffers: Vec<AudioBuffer<S>>,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
}

impl<S: Sample> Signal<S> {
    pub fn new(graph: Graph<S>) -> Self {
        let channels = graph.context().num_channels;
        let buffer_size = graph.context().buffer_size;
        let sample_rate = graph.context().sample_rate as u32;

        let mut output_buffers = Vec::with_capacity(channels);
        for _ in 0..channels {
            output_buffers.push(AudioBuffer::new(buffer_size));
        }

        Self {
            graph,
            output_buffers,
            channel_index: 0,
            sample_index: 0,
            num_channels: channels,
            buffer_size,
            sample_rate,
        }
    }

    fn process(&mut self) -> S {
        if self.channel_index == 0 && self.sample_index == 0 {
            let mut output_refs: Vec<&mut [S]> = self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
            self.graph.process_buffers(&mut output_refs);
        }

        let sample = self.output_buffers[self.channel_index][self.sample_index];

        // `rodio` expects interleaved samples, so we have to increment the
        // channel index every time and only increment the sample index when the
        // channel index has to be wrapped around.
        self.channel_index += 1;
        if self.channel_index >= self.num_channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl<S: Sample> Iterator for Signal<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for Signal<f32> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.num_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
