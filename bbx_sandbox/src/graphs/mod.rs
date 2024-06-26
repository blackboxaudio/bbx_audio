use bbx_dsp::graph::Graph;

mod sine_wave;
mod sine_wave_with_overdrive;

#[allow(dead_code)]
pub enum GraphType {
    SineWave,
    SineWaveWithOverdrive,
}

pub fn get_graph_from_type(sample_rate: usize, graph_type: GraphType) -> Graph {
    let mut graph = Graph::new(sample_rate);
    match graph_type {
        GraphType::SineWave => sine_wave::create_sine_wave(&mut graph),
        GraphType::SineWaveWithOverdrive => sine_wave_with_overdrive::create_sine_wave_with_overdrive(&mut graph),
    }

    return graph;
}
