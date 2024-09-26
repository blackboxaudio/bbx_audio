/// The context in which a DSP graph should be evaluated, containing
/// information such as how many channels should be generated or processed or what
/// size should the buffers be.
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
