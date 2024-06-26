use bbx_dsp::graph::Graph;

pub fn create_simple_synth(sample_rate: usize) -> Graph {
    return Graph::new(sample_rate);
}
