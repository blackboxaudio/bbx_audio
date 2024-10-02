use bbx_buffer::buffer::{AudioBuffer, Buffer};
use bbx_dsp::{
    constants::DEFAULT_CONTEXT,
    graph::Graph,
    node::Node,
    operation::OperationType,
    process::{AudioInput, Process},
};
use bbx_sandbox::{player::Player, signal::Signal};
use rand::Rng;

pub struct NoiseGenerator;

impl Process for NoiseGenerator {
    fn process(&mut self, _inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        let amplitude_multiplier: f32 = 0.1;
        let mut rng = rand::thread_rng();
        for channel_buffer in output.iter_mut() {
            channel_buffer.apply_mut(|_s| {
                let random_value: f32 = rng.gen();
                random_value * amplitude_multiplier
            })
        }
    }
}

fn main() {
    // Create a `Graph` with the default context, add the custom noise generator,
    // and prepare it for playback
    let mut graph = Graph::new(DEFAULT_CONTEXT);
    graph.add_node(
        Node::new(DEFAULT_CONTEXT, Box::new(NoiseGenerator), OperationType::Generator),
        None,
    );
    graph.prepare_for_playback();

    // Play a `Signal` created from the graph
    let signal = Signal::new(graph);
    Player::new(signal).play(None);
}
