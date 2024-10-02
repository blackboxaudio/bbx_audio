use bbx_buffer::buffer::AudioBuffer;

/// Implemented by `Node` objects used in a DSP `Graph` to process or generate signals.
pub trait Process {
    fn process(&mut self, inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]);
}

/// A pointer to the output buffers of another node that is an input
/// to the current node.
#[derive(Debug)]
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

    pub fn as_slice(&self) -> &[AudioBuffer<f32>] {
        // Because an `AudioInput` is only instantiated during the evaluation of a `Graph`,
        // we know that the slice is valid for as long as this input is alive.
        unsafe { std::slice::from_raw_parts(self.buffers_ptr, self.buffers_len) }
    }
}
