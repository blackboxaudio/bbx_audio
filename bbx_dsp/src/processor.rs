use petgraph::data::{DataMap, DataMapMut};
use petgraph::Incoming;
use petgraph::prelude::DfsPostOrder;
use petgraph::visit::{Data, GraphBase, IntoNeighborsDirected, Reversed, Visitable};
use crate::node::{AudioInput, Node, NodeData};

/// Evaluates and contains state of an audio graph `G`.
pub struct Processor<G: Visitable> {
    evaluation_order: DfsPostOrder<G::NodeId, G::Map>,
    inputs: Vec<AudioInput>,
}

impl<G: Visitable> Processor<G> {
    pub fn with_capacity(max_num_nodes: usize) -> Self where G::Map: Default {
        let mut evaluation_order = DfsPostOrder::default();
        evaluation_order.stack = Vec::with_capacity(max_num_nodes);
        let inputs= Vec::with_capacity(max_num_nodes);
        Processor {
            evaluation_order,
            inputs,
        }
    }

    pub fn process<N>(&mut self, graph: &mut G, node_id: G::NodeId)
    where
        G: Data<NodeWeight = NodeData<N>> + DataMapMut,
        for<'a> &'a G: GraphBase<NodeId = G::NodeId> + IntoNeighborsDirected,
        N: Node,
    {
        process_graph(self, graph, node_id);
    }
}

pub fn process_graph<G, N>(processor: &mut Processor<G>, graph: &mut G, node_id: G::NodeId)
where
    G: Data<NodeWeight = NodeData<N>> + DataMapMut + Visitable,
    for<'a> &'a G: GraphBase<NodeId = G::NodeId> + IntoNeighborsDirected,
    N: Node,
{
    const NO_NODE_EXISTS: &str = "no node exists for the given ID";

    processor.evaluation_order.reset(Reversed(&*graph));
    processor.evaluation_order.move_to(node_id);

    while let Some(n_id) = processor.evaluation_order.next(Reversed(&*graph)) {
        let data: *mut NodeData<N> = graph.node_weight_mut(n_id).expect(NO_NODE_EXISTS) as *mut _;
        processor.inputs.clear();
        for in_node_id in (&*graph).neighbors_directed(n_id, Incoming) {
            if n_id == in_node_id {
                continue;
            }

            let input_node_data = graph.node_weight(in_node_id).expect(NO_NODE_EXISTS);
            let input = AudioInput::new(&input_node_data.buffers);
            processor.inputs.push(input);
        }

        unsafe {
            (*data).node.process(&processor.inputs, &mut (*data).buffers);
        }
    }
}
