use bbx_dsp::{block::Block, effector::Effector, generator::Generator, graph::Graph};

pub fn create_sine_wave_with_overdrive(graph: &mut Graph) {
    let mut oscillator = Generator::new(graph.sample_rate());
    oscillator.set_frequency(220.0);
    graph.add_block(Block::Generator(oscillator));

    let num_overdrive_effectors: i32 = 24;
    for _ in 0..num_overdrive_effectors {
        graph.add_block(Block::Effector(Effector::new()));
    }
}
