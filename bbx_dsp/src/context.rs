/// The context in which a DSP graph should be evaluated, containing
/// information such as how many channels should be generated or processed or what
/// size should the buffers be.
pub struct Context {
    /// The amount of samples being generated or processed per second.
    pub sample_rate: usize,

    /// The number of samples within the audio buffers.
    pub buffer_size: usize,

    /// The number of audio channels each containing their own audio buffer.
    pub num_channels: usize,
}

impl Context {
    pub fn new(sample_rate: usize, buffer_size: usize, num_channels: usize) -> Self {
        Context { sample_rate, buffer_size, num_channels }
    }
}
