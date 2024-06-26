use bbx_dsp::graph::Graph;

mod simple_synth;

pub enum GraphType {
    SimpleOsc,
}

pub fn get_graph_from_type(sample_rate: usize, graph_type: GraphType) -> Graph {
    let mut graph = Graph::new(sample_rate);
    match graph_type {
        GraphType::SimpleOsc => simple_synth::create_simple_synth(&mut graph),
    }

    return graph;
}
