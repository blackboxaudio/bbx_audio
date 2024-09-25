use std::marker::PhantomData;
use petgraph::data::DataMapMut;
use petgraph::visit::{Data, GraphBase, IntoNeighborsDirected, Visitable};
use crate::buffer::{AudioBuffer, Buffer};
use crate::node::{AudioInput, Node, NodeData};
use crate::processor::Processor;

pub struct Graph<G, N>
where
    G: Visitable,
{
    pub processor: Processor<G>,
    pub graph_node: G,
    pub input_nodes: Vec<G::NodeId>,
    pub output_node: G::NodeId,
    pub node_type: PhantomData<N>,
}

impl <G, N> Node for Graph<G, N>
where
    G: Data<NodeWeight = NodeData<N>> + DataMapMut + Visitable,
    for<'a> &'a G: GraphBase<NodeId = G::NodeId> + IntoNeighborsDirected,
    N: Node,
{
    fn process(&mut self, audio_inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        let Graph {
            ref mut processor,
            ref mut graph_node,
            ref input_nodes,
            output_node,
            ..
        } = *self;

        for (audio_input, &input_node_id) in audio_inputs.iter().zip(input_nodes) {
            let input_node_buffers = &mut graph_node.node_weight_mut(input_node_id).expect("no node for given ID").buffers;
            for (input_node_buffer, input_buffer) in input_node_buffers.iter_mut().zip(audio_input.buffers()) {
                input_node_buffer.copy_from_slice(input_buffer.as_slice());
            }
        }

        processor.process(graph_node, output_node);

        let output_node_buffers = &mut graph_node.node_weight_mut(output_node).expect("no node for graph output node").buffers;
        for (output_buffer, output_node_buffer) in output.iter_mut().zip(output_node_buffers) {
            output_buffer.copy_from_slice(output_node_buffer.as_slice());
        }
    }
}
