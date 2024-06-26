use bbx_dsp::block::Block;
use bbx_dsp::generator::Generator;
use bbx_dsp::graph::Graph;

pub fn create_simple_synth(graph: &mut Graph) {
    let mut oscillator = Generator::new(graph.sample_rate());
    oscillator.set_frequency(220.0);
    graph.add_block(Block::Generator(oscillator));
}
