use std::collections::HashMap;

use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    context::Context,
    effector::Effector,
    error::BbxAudioDspError,
    generator::Generator,
    node::{Node, NodeId},
    operation::OperationType,
    process::AudioInput,
};

/// Contains a number of `Node`s connected in a certain way.
pub struct Graph {
    /// The associated `Context` for this `Graph`.
    pub context: Context,

    nodes: HashMap<NodeId, Node>,
    connections: Vec<(NodeId, NodeId)>,

    processes: HashMap<NodeId, Vec<AudioBuffer<f32>>>,
    processing_order: Vec<NodeId>,
}

impl Graph {
    pub fn new(context: Context) -> Graph {
        let max_num_graph_nodes = context.max_num_graph_nodes;
        Graph {
            context,
            nodes: HashMap::with_capacity(max_num_graph_nodes),
            connections: Vec::with_capacity(max_num_graph_nodes),
            processes: HashMap::with_capacity(max_num_graph_nodes),
            processing_order: Vec::with_capacity(max_num_graph_nodes),
        }
    }
}

impl Graph {
    /// Adds a `Node` to the graph
    pub fn add_node(&mut self, node: Node, error: Option<BbxAudioDspError>) -> NodeId {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        self.processes.insert(
            node_id,
            vec![AudioBuffer::new(self.context.buffer_size); self.context.num_channels],
        );

        if let Some(node) = self.nodes.get(&node_id) {
            node.id
        } else {
            panic!("{:?}", error.unwrap_or(BbxAudioDspError::CannotAddNode));
        }
    }

    /// Adds an `Effector` to the graph
    pub fn add_effector(&mut self, effector: Effector) -> NodeId {
        let effector_node = Node::from_effector(self.context, effector);
        self.add_node(effector_node, Some(BbxAudioDspError::CannotAddEffectorNode))
    }

    /// Adds a `Generator` to the graph
    pub fn add_generator(&mut self, generator: Generator) -> NodeId {
        let generator_node = Node::from_generator(self.context, generator);
        self.add_node(generator_node, Some(BbxAudioDspError::CannotAddGeneratorNode))
    }

    /// Creates a connection between a source node and destination node.
    pub fn create_connection(&mut self, source_id: NodeId, destination_id: NodeId) {
        if self.connections.contains(&(source_id, destination_id)) {
            panic!("{:?}", BbxAudioDspError::ConnectionAlreadyCreated);
        } else {
            if let Some(source) = self.nodes.get_mut(&source_id) {
                source.add_output(destination_id);
            } else {
                panic!(
                    "{:?}",
                    BbxAudioDspError::CannotRetrieveSourceNode(format!("{}", source_id))
                );
            }
            if let Some(destination) = self.nodes.get_mut(&destination_id) {
                destination.add_input(source_id);
            } else {
                panic!(
                    "{:?}",
                    BbxAudioDspError::CannotRetrieveDestinationNode(format!("{}", destination_id))
                );
            }
            self.connections.push((source_id, destination_id));
        }
    }

    /// Prepares a `Graph` to be processed, i.e. ensures optimal node evaluation order,
    /// validates acyclicity, and checks that all connections are valid.
    pub fn prepare_for_playback(&mut self) {
        self.update_processing_order();
        self.validate_acyclicity();
        self.validate_connections();
        self.validate_convergence();
    }
}

impl Graph {
    fn update_processing_order(&mut self) {
        let mut stack: Vec<NodeId> = Vec::with_capacity(self.nodes.len());
        let mut visited: Vec<NodeId> = Vec::with_capacity(self.nodes.len());

        fn dfs(node: &Node, order: &mut Vec<NodeId>, visited: &mut Vec<NodeId>, nodes: &HashMap<NodeId, Node>) {
            visited.push(node.id);
            for &node_id in &node.outputs {
                if visited.contains(&node_id) {
                    continue;
                } else {
                    let node_option = nodes.get(&node_id);
                    if let Some(node) = node_option {
                        dfs(node, order, visited, nodes);
                    }
                }
            }
            order.push(node.id);
        }

        for node in self.nodes.values() {
            if visited.contains(&node.id) {
                continue;
            } else {
                dfs(node, &mut stack, &mut visited, &self.nodes);
            }
        }

        if stack.len() == self.nodes.len() {
            stack.reverse();
            self.processing_order = stack.clone();
        } else {
            panic!("{:?}", BbxAudioDspError::CannotUpdateGraphProcessingOrder);
        }
    }

    fn validate_acyclicity(&self) {
        fn dfs(original_node_id: NodeId, node: &Node, visited: &mut Vec<NodeId>, nodes: &HashMap<NodeId, Node>) {
            visited.push(node.id);
            for &node_id in &node.outputs {
                if visited.contains(&node_id) {
                    if node_id == original_node_id {
                        panic!("{:?}", BbxAudioDspError::GraphContainsCycle(format!("{}", node_id)))
                    }
                    continue;
                } else {
                    let node_option = nodes.get(&node_id);
                    if let Some(node) = node_option {
                        dfs(original_node_id, node, visited, nodes);
                    }
                }
            }
        }

        for node in self.nodes.values() {
            let mut visited: Vec<NodeId> = Vec::with_capacity(self.nodes.len());
            dfs(node.id, node, &mut visited, &self.nodes);
        }
    }

    fn validate_connections(&self) {
        for (source_id, destination_id) in &self.connections {
            if self.nodes.contains_key(source_id) && self.nodes.contains_key(destination_id) {
                continue;
            } else {
                panic!("{:?}", BbxAudioDspError::ConnectionHasNoNode);
            }
        }
        for (node_id, node) in self.nodes.iter() {
            if node.operation_type == OperationType::Effector && node.inputs.is_empty() {
                panic!("{:?}", BbxAudioDspError::NodeHasNoInputs(format!("{}", node_id)));
            } else if node.operation_type == OperationType::Generator && node.outputs.is_empty() && self.nodes.len() > 1
            {
                panic!("{:?}", BbxAudioDspError::NodeHasNoOutputs(format!("{}", node_id)));
            }
        }
    }

    fn validate_convergence(&self) {
        fn dfs(_original_node_id: NodeId, node: &Node, visited: &mut Vec<NodeId>, nodes: &HashMap<NodeId, Node>) {
            visited.push(node.id);
            for &node_id in &node.inputs {
                if visited.contains(&node_id) {
                    continue;
                } else {
                    let node_option = nodes.get(&node_id);
                    if let Some(node) = node_option {
                        dfs(_original_node_id, node, visited, nodes);
                    }
                }
            }
        }

        let node_id = self.processing_order.last().unwrap();
        let node = self.nodes.get(node_id).unwrap();
        let mut visited: Vec<NodeId> = Vec::with_capacity(self.nodes.len());
        dfs(*node_id, node, &mut visited, &self.nodes);

        if self.nodes.len() != visited.len() {
            panic!("{:?}", BbxAudioDspError::GraphContainsNonConvergingPaths);
        }
    }
}

impl Graph {
    /// Iterates through the nodes of a graph and processes each of them.
    #[allow(unused_assignments)]
    pub fn evaluate(&mut self) -> &Vec<AudioBuffer<f32>> {
        for &node_id in &self.processing_order {
            let node = self.nodes.get_mut(&node_id).unwrap();
            let inputs = &node
                .inputs
                .iter()
                .map(|i| AudioInput::new(self.processes.get(i).unwrap().as_slice()))
                .collect::<Vec<AudioInput>>()[..];
            node.operation
                .process(inputs, self.processes.get_mut(&node_id).unwrap());
        }

        self.processes.get(self.processing_order.last().unwrap()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;
    use crate::{context::Context, effector::Effector, generator::Generator, generators::wave_table::Waveform};

    #[test]
    fn test_graph_creation() {
        let context = Context::new(1024, 7, 16, 32);
        let graph = Graph::new(context);

        assert_eq!(graph.context.sample_rate, 1024);
        assert_eq!(graph.context.num_channels, 7);
        assert_eq!(graph.context.buffer_size, 16);
        assert!(graph.nodes.is_empty());
        assert!(graph.connections.is_empty());
        assert!(graph.processes.is_empty());
    }

    #[test]
    fn test_add_effector() {
        let context = Context::new(1024, 2, 4, 32);
        let mut graph = Graph::new(context);
        let effector_id = graph.add_effector(Effector::Overdrive);
        assert!(graph.nodes.contains_key(&effector_id));
    }

    #[test]
    fn test_add_generator() {
        let context = Context::new(1024, 2, 4, 32);
        let mut graph = Graph::new(context);
        let generator_id = graph.add_generator(Generator::WaveTable {
            frequency: 110.0,
            waveform: Waveform::Sine,
        });
        assert!(graph.nodes.contains_key(&generator_id));
    }

    #[test]
    fn test_create_connection() {
        let context = Context::new(1024, 2, 4, 32);
        let mut graph = Graph::new(context);

        let osc = graph.add_generator(Generator::WaveTable {
            frequency: 110.0,
            waveform: Waveform::Sine,
        });
        let overdrive = graph.add_effector(Effector::Overdrive);
        graph.create_connection(osc, overdrive);

        assert!(graph.connections.contains(&(osc, overdrive)));
    }

    #[test]
    #[should_panic(expected = "ConnectionAlreadyCreated")]
    fn test_duplicate_connection() {
        let context = Context::new(1024, 2, 4, 32);
        let mut graph = Graph::new(context);

        let osc = graph.add_generator(Generator::WaveTable {
            frequency: 110.0,
            waveform: Waveform::Sine,
        });
        let overdrive = graph.add_effector(Effector::Overdrive);
        graph.create_connection(osc, overdrive);
        graph.create_connection(osc, overdrive); // This should panic
    }

    #[test]
    fn test_prepare_for_playback() {
        let context = Context::new(1024, 2, 4, 32);
        let mut graph = Graph::new(context);

        let osc = graph.add_generator(Generator::WaveTable {
            frequency: 110.0,
            waveform: Waveform::Sine,
        });
        let overdrive = graph.add_effector(Effector::Overdrive);
        graph.create_connection(osc, overdrive);
        graph.prepare_for_playback();

        assert_eq!(graph.processing_order.len(), 2);
    }

    #[test]
    fn test_evaluate() {
        let context = Context::new(1024, 6, 4, 32);
        let mut graph = Graph::new(context);

        let osc = graph.add_generator(Generator::WaveTable {
            frequency: 110.0,
            waveform: Waveform::Sine,
        });
        let overdrive = graph.add_effector(Effector::Overdrive);
        graph.create_connection(osc, overdrive);
        graph.prepare_for_playback();

        let result = graph.evaluate();
        assert_eq!(result.len(), 6); // Assuming 2 channels for this context
    }

    #[bench]
    fn bench_evaluate(b: &mut Bencher) {
        let context = Context::new(44100, 2, 128, 2048);
        let mut graph = Graph::new(context);

        let mixer = graph.add_effector(Effector::Mixer);
        for n in 0..256 {
            let osc = graph.add_generator(Generator::WaveTable {
                frequency: 110.0 * (n + 1) as f32,
                waveform: Waveform::Sine,
            });
            graph.create_connection(osc, mixer);
        }
        let overdrive = graph.add_effector(Effector::Overdrive);
        graph.create_connection(mixer, overdrive);

        graph.prepare_for_playback();

        b.iter(|| {
            graph.evaluate();
        });
    }
}
