use std::collections::HashMap;
use crate::{
    block::Block,
    buffer::{AudioBuffer, Buffer},
    context::Context,
    effector::Effector,
    error::BbxAudioDspError,
    generator::Generator,
    node::NodeId,
    operation::OperationType,
    process::AudioInput,
};

/// Contains a number of `Block`s connected in a certain way.
pub struct Graph {
    /// The associated `Context` for this `Graph`.
    pub context: Context,

    blocks: HashMap<NodeId, Block>,
    connections: Vec<(NodeId, NodeId)>,

    processes: HashMap<NodeId, Vec<AudioBuffer<f32>>>,
    processing_order: Vec<NodeId>,
}

impl Graph {
    pub fn new(context: Context) -> Graph {
        let max_num_graph_nodes = context.max_num_graph_nodes;
        Graph {
            context,
            blocks: HashMap::with_capacity(max_num_graph_nodes),
            connections: Vec::with_capacity(max_num_graph_nodes),
            processes: HashMap::with_capacity(max_num_graph_nodes),
            processing_order: Vec::with_capacity(max_num_graph_nodes),
        }
    }
}

impl Graph {
    /// Adds an `Effector` to the graph
    pub fn add_effector(&mut self, effector: Effector) -> usize {
        let effector_block = Block::from_effector_operation(self.context, effector.to_operation());
        self.add_block(effector_block, BbxAudioDspError::CannotAddEffectorBlock)
    }

    /// Adds a `Generator` to the graph
    pub fn add_generator(&mut self, generator: Generator) -> usize {
        let generator_block = Block::from_generator(self.context, generator);
        self.add_block(generator_block, BbxAudioDspError::CannotAddGeneratorBlock)
    }

    fn add_block(&mut self, block: Block, error: BbxAudioDspError) -> usize {
        let block_id = block.id;
        self.blocks.insert(block_id, block);
        self.processes.insert(
            block_id,
            vec![AudioBuffer::new(self.context.buffer_size); self.context.num_channels],
        );

        if let Some(block) = self.blocks.get(&block_id) {
            block.id
        } else {
            panic!("{:?}", error);
        }
    }

    /// Creates a connection between a source node and destination node.
    pub fn create_connection(&mut self, source_id: NodeId, destination_id: NodeId) {
        if self.connections.contains(&(source_id, destination_id)) {
            panic!("{:?}", BbxAudioDspError::ConnectionAlreadyCreated);
        } else {
            if let Some(source) = self.blocks.get_mut(&source_id) {
                source.add_output(destination_id);
            } else {
                panic!(
                    "{:?}",
                    BbxAudioDspError::CannotRetrieveSourceBlock(format!("{}", source_id))
                );
            }
            if let Some(destination) = self.blocks.get_mut(&destination_id) {
                destination.add_input(source_id);
            } else {
                panic!(
                    "{:?}",
                    BbxAudioDspError::CannotRetrieveDestinationBlock(format!("{}", destination_id))
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
        let mut stack: Vec<NodeId> = Vec::with_capacity(self.blocks.len());
        let mut visited: Vec<NodeId> = Vec::with_capacity(self.blocks.len());

        fn dfs(block: &Block, order: &mut Vec<NodeId>, visited: &mut Vec<NodeId>, blocks: &HashMap<NodeId, Block>) {
            visited.push(block.id);
            for &block_id in &block.outputs {
                if visited.contains(&block_id) {
                    continue;
                } else {
                    let block_option = blocks.get(&block_id);
                    if let Some(block) = block_option {
                        dfs(block, order, visited, blocks);
                    }
                }
            }
            order.push(block.id);
        }

        for block in self.blocks.values() {
            if visited.contains(&block.id) {
                continue;
            } else {
                dfs(block, &mut stack, &mut visited, &self.blocks);
            }
        }

        if stack.len() == self.blocks.len() {
            stack.reverse();
            self.processing_order = stack.clone();
        } else {
            panic!("{:?}", BbxAudioDspError::CannotUpdateGraphProcessingOrder);
        }
    }

    fn validate_acyclicity(&self) {
        fn dfs(original_block_id: NodeId, block: &Block, visited: &mut Vec<NodeId>, blocks: &HashMap<NodeId, Block>) {
            visited.push(block.id);
            for &block_id in &block.outputs {
                if visited.contains(&block_id) {
                    if block_id == original_block_id {
                        panic!("{:?}", BbxAudioDspError::GraphContainsCycle(format!("{}", block_id)))
                    }
                    continue;
                } else {
                    let block_option = blocks.get(&block_id);
                    if let Some(block) = block_option {
                        dfs(original_block_id, block, visited, blocks);
                    }
                }
            }
        }

        for block in self.blocks.values() {
            let mut visited: Vec<NodeId> = Vec::with_capacity(self.blocks.len());
            dfs(block.id, block, &mut visited, &self.blocks);
        }
    }

    fn validate_connections(&self) {
        for (source_id, destination_id) in &self.connections {
            if self.blocks.contains_key(source_id) && self.blocks.contains_key(destination_id) {
                continue;
            } else {
                panic!("{:?}", BbxAudioDspError::ConnectionHasNoBlock);
            }
        }
        for (block_id, block) in self.blocks.iter() {
            if block.operation_type == OperationType::Effector && block.inputs.is_empty() {
                panic!("{:?}", BbxAudioDspError::BlockHasNoInputs(format!("{}", block_id)));
            } else if block.operation_type == OperationType::Generator
                && block.outputs.is_empty()
                && self.blocks.len() > 1
            {
                panic!("{:?}", BbxAudioDspError::BlockHasNoOutputs(format!("{}", block_id)));
            }
        }
    }

    fn validate_convergence(&self) {
        fn dfs(original_block_id: NodeId, block: &Block, visited: &mut Vec<NodeId>, blocks: &HashMap<NodeId, Block>) {
            visited.push(block.id);
            for &block_id in &block.inputs {
                if visited.contains(&block_id) {
                    continue;
                } else {
                    let block_option = blocks.get(&block_id);
                    if let Some(block) = block_option {
                        dfs(original_block_id, block, visited, blocks);
                    }
                }
            }
        }

        let block_id = self.processing_order.last().unwrap();
        let block = self.blocks.get(block_id).unwrap();
        let mut visited: Vec<NodeId> = Vec::with_capacity(self.blocks.len());
        dfs(*block_id, block, &mut visited, &self.blocks);

        if self.blocks.len() != visited.len() {
            panic!("{:?}", BbxAudioDspError::GraphContainsNonConvergingPaths);
        }
    }
}

impl Graph {
    /// Iterates through the nodes of a graph and processes each of them.
    #[allow(unused_assignments)]
    pub fn evaluate(&mut self) -> &Vec<AudioBuffer<f32>> {
        for &block_id in &self.processing_order {
            let block = self.blocks.get_mut(&block_id).unwrap();
            let inputs = &block
                .inputs
                .iter()
                .map(|i| AudioInput::new(self.processes.get(i).unwrap()))
                .collect::<Vec<AudioInput>>()[..];
            block
                .operation
                .process(inputs, self.processes.get_mut(&block_id).unwrap());

            let output = self.processes.get(&block_id).unwrap();
            self.processes.insert(block_id, output.to_vec());
        }

        self.processes.get(self.processing_order.last().unwrap()).unwrap()
    }
}
