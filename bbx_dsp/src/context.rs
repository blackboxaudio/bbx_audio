use crate::constants::DEFAULT_CONTEXT;

/// The context in which a DSP graph should be evaluated, containing
/// information such as how many channels should be generated or processed or what
/// size should the buffers be.
#[derive(Clone, Copy)]
pub struct Context {
    /// The rate at which samples will be generated per second.
    pub sample_rate: usize,

    /// The number of channels that will each contain a buffer of audio samples.
    pub num_channels: usize,

    /// The number of audio samples to include in a buffer.
    pub buffer_size: usize,

    /// The maximum number of nodes a `Graph` can contain.
    pub max_num_graph_nodes: usize,
}

impl Context {
    pub fn new(sample_rate: usize, num_channels: usize, buffer_size: usize, max_num_graph_nodes: usize) -> Self {
        Context {
            sample_rate,
            num_channels,
            buffer_size,
            max_num_graph_nodes,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        DEFAULT_CONTEXT
    }
}
