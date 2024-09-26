use crate::context::Context;

pub const DEFAULT_CONTEXT: Context = Context {
    sample_rate: 44100,
    buffer_size: 256,
    max_num_graph_nodes: 384,
};
