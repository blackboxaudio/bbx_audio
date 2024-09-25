#[derive(Clone, Copy)]
pub struct Context {
    pub sample_rate: usize,
    pub buffer_size: usize,
    pub max_num_graph_nodes: usize,
}

impl Context {
    pub fn new(sample_rate: usize, buffer_size: usize, max_num_graph_nodes: usize) -> Self {
        Context {
            sample_rate,
            buffer_size,
            max_num_graph_nodes,
        }
    }
}
