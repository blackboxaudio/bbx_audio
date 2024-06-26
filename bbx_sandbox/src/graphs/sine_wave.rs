use bbx_dsp::{block::Block, generator::Generator, graph::Graph};

pub fn create_sine_wave(graph: &mut Graph) {
    let mut oscillator = Generator::new(graph.sample_rate());
    oscillator.set_frequency(220.0);
    graph.add_block(Block::Generator(oscillator));
}
