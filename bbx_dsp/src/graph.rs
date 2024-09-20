use std::collections::HashMap;

use crate::{
    block::Block, effector::Effector, error::BbxAudioDspError, generator::Generator, operation::OperationType,
    sample::Sample,
};

/// A collection of interconnected `Block` objects.
pub struct Graph {
    sample_rate: usize,
    blocks: HashMap<usize, Block>,
    connections: Vec<(usize, usize)>,
    processes: HashMap<usize, Sample>,
    processing_order: Vec<usize>,
}

impl Graph {
    pub fn new(sample_rate: usize) -> Graph {
        return Graph {
            sample_rate,
            blocks: HashMap::new(),
            connections: Vec::new(),
            processes: HashMap::new(),
            processing_order: Vec::new(),
        };
    }

    pub fn sample_rate(&self) -> usize {
        return self.sample_rate;
    }
}

impl Graph {
    pub fn add_effector(&mut self, effector: Effector) -> usize {
        let effector_block = Block::from_effector_operation(effector.to_operation());
        return self.add_block(effector_block, BbxAudioDspError::CannotAddEffectorBlock);
    }

    pub fn add_generator(&mut self, generator: Generator) -> usize {
        let generator_block = Block::from_generator(generator);
        return self.add_block(generator_block, BbxAudioDspError::CannotAddGeneratorBlock);
    }

    fn add_block(&mut self, block: Block, error: BbxAudioDspError) -> usize {
        let block_id = block.id;
        self.blocks.insert(block_id, block);
        self.processes.insert(block_id, 0.0);

        return if let Some(block) = self.blocks.get(&block_id) {
            block.id
        } else {
            panic!("{:?}", error);
        };
    }

    pub fn create_connection(&mut self, source_id: usize, destination_id: usize) {
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

    pub fn prepare_for_playback(&mut self) {
        self.update_processing_order();
        self.validate_acyclicity();
        self.validate_connections();
    }
}

impl Graph {
    fn update_processing_order(&mut self) {
        let mut stack: Vec<usize> = Vec::with_capacity(self.blocks.len());
        let mut visited: Vec<usize> = Vec::with_capacity(self.blocks.len());

        fn dfs(block: &Block, order: &mut Vec<usize>, visited: &mut Vec<usize>, blocks: &HashMap<usize, Block>) {
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

        for (_, block) in &self.blocks {
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
        fn dfs(original_block_id: usize, block: &Block, visited: &mut Vec<usize>, blocks: &HashMap<usize, Block>) {
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

        for (_, block) in &self.blocks {
            let mut visited: Vec<usize> = Vec::with_capacity(self.blocks.len());
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
            if block.operation_type == OperationType::Effector && block.inputs.len() == 0 {
                panic!("{:?}", BbxAudioDspError::BlockHasNoInputs(format!("{}", block_id)));
            } else if block.operation_type == OperationType::Generator
                && block.outputs.len() == 0
                && self.blocks.len() > 1
            {
                panic!("{:?}", BbxAudioDspError::BlockHasNoOutputs(format!("{}", block_id)));
            }
        }
    }
}

impl Graph {
    #[allow(unused_assignments)]
    pub fn evaluate(&mut self) -> Sample {
        for &block_id in &self.processing_order {
            let block = self.blocks.get_mut(&block_id).unwrap();
            let mut inputs: Vec<Sample> = Vec::with_capacity(block.inputs.len());
            for input in &block.inputs {
                inputs.push(*self.processes.get(input).unwrap());
            }
            self.processes.insert(block_id, block.operation.process(&inputs));
        }

        return *self
            .processes
            .get(self.processing_order.last().unwrap())
            .unwrap_or_else(|| &0.0);
    }
}
