use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bbx_core::StackVec;
use bbx_dsp::{
    buffer::{AudioBuffer, Buffer},
    graph::{Graph, MAX_BLOCK_OUTPUTS},
    sample::Sample,
};

/// Wraps a DSP graph as an iterator of interleaved samples.
///
/// The iterator yields samples in interleaved format (L, R, L, R, ...)
/// which is the format expected by audio backends.
pub struct Signal<S: Sample> {
    graph: Graph<S>,
    output_buffers: Vec<AudioBuffer<S>>,
    sample_rate: u32,
    num_channels: usize,
    buffer_size: usize,
    channel_index: usize,
    sample_index: usize,
    stop_flag: Arc<AtomicBool>,
}

impl<S: Sample> Signal<S> {
    /// Create a `Signal` from a DSP `Graph`.
    pub fn new(graph: Graph<S>, stop_flag: Arc<AtomicBool>) -> Self {
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
            stop_flag,
        }
    }

    /// Get the sample rate of the signal.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels.
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn process(&mut self) -> S {
        if self.channel_index == 0 && self.sample_index == 0 {
            debug_assert!(
                self.num_channels <= MAX_BLOCK_OUTPUTS,
                "Channel count {} exceeds MAX_BLOCK_OUTPUTS {}",
                self.num_channels,
                MAX_BLOCK_OUTPUTS
            );
            let mut output_refs: StackVec<&mut [S], MAX_BLOCK_OUTPUTS> = StackVec::new();
            for buf in self.output_buffers.iter_mut() {
                let _ = output_refs.push(buf.as_mut_slice());
            }
            self.graph.process_buffers(output_refs.as_mut_slice());
        }

        // SAFETY: Bounds guaranteed by construction:
        // - channel_index: starts at 0, reset to 0 when >= num_channels (line 80)
        // - sample_index: wraps via modulo % buffer_size (line 83)
        debug_assert!(
            self.channel_index < self.num_channels,
            "channel_index {} exceeds num_channels {}",
            self.channel_index,
            self.num_channels
        );
        debug_assert!(
            self.sample_index < self.buffer_size,
            "sample_index {} exceeds buffer_size {}",
            self.sample_index,
            self.buffer_size
        );
        let sample = self.output_buffers[self.channel_index][self.sample_index];

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
        if self.stop_flag.load(Ordering::SeqCst) {
            return None;
        }
        Some(self.process())
    }
}
