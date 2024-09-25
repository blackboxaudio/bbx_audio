use bbx_dsp::context::Context;

pub const PLAYTIME_DURATION: usize = 3;

pub const SAMPLE_RATE: usize = 44100;
pub const BUFFER_SIZE: usize = 256;

pub const DEFAULT_CONTEXT: Context = Context {
    sample_rate: SAMPLE_RATE,
    buffer_size: BUFFER_SIZE,
    max_num_graph_nodes: 384,
};
