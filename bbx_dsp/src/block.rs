use rand::Rng;

use crate::{
    context::Context,
    generator::Generator,
    node::NodeId,
    operation::{Operation, OperationType},
};

/// Represents an implementation that generates audio output buffers from a number of audio inputs.
pub struct Block {
    pub id: NodeId,

    pub inputs: Vec<NodeId>,
    pub outputs: Vec<NodeId>,

    pub operation: Operation,
    pub operation_type: OperationType,
}

impl Block {
    fn new(operation: Operation, operation_type: OperationType) -> Block {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<NodeId>();
        Block {
            id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            operation,
            operation_type,
        }
    }

    pub fn from_effector_operation(effector_operation: Operation) -> Block {
        Self::new(effector_operation, OperationType::Effector)
    }

    pub fn from_generator(context: Context, generator: Generator) -> Block {
        Self::new(generator.to_operation(context), OperationType::Generator)
    }
}

impl Block {
    pub fn add_output(&mut self, output: NodeId) {
        self.outputs.push(output);
    }

    pub fn add_input(&mut self, input: NodeId) {
        self.inputs.push(input);
    }
}
