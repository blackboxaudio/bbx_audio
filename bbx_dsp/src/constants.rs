use crate::context::Context;

/// The default context in which to evaluate a `Graph`.
pub const DEFAULT_CONTEXT: Context = Context {
    sample_rate: 44100,
    num_channels: 2,
    buffer_size: 256,
    max_num_graph_nodes: 384,
};
