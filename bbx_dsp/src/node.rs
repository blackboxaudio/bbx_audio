use crate::buffer::AudioBuffer;

/// The identifier of a node within the DSP graph.
pub type NodeId = usize;

/// Represents an implementation that generates an audio output buffer from a number of audio inputs.
pub trait Node {
    /// Writes to a given slice of audio buffers, while optionally processing audio and modulation inputs.
    fn process(&mut self, audio_inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]);
}

/// A pointer to the output buffers of another node that is an input
/// to the current node.
pub struct AudioInput {
    buffers_ptr: *const AudioBuffer<f32>,
    buffers_len: usize,
}

impl AudioInput {
    pub fn new(slice: &[AudioBuffer<f32>]) -> Self {
        let buffers_ptr = slice.as_ptr();
        let buffers_len = slice.len();
        AudioInput {
            buffers_ptr,
            buffers_len,
        }
    }

    pub fn buffers(&self) -> &[AudioBuffer<f32>] {
        // Because an `AudioInput` is only instantiated during the evaluation of a `Graph`
        // by a `Processor`, we know that the slice is valid for as long as this input is alive.
        unsafe { std::slice::from_raw_parts(self.buffers_ptr, self.buffers_len) }
    }
}
