use std::fmt::{Display, Formatter};

use rand::Rng;

use crate::{
    generator::Generator,
    operation::{Operation, OperationType},
};
use crate::node::NodeId;

/// Represents an implementation that generates audio output buffers from a number of audio inputs.
pub struct Block {
    pub id: NodeId,

    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,

    pub operation: Operation,
    pub operation_type: OperationType,
}

impl Block {
    fn new(operation: Operation, operation_type: OperationType) -> Block {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<usize>();
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

    pub fn from_generator(generator: Generator) -> Block {
        Self::new(generator.to_operation(), OperationType::Generator)
    }
}

impl Block {
    pub fn add_output(&mut self, output: usize) {
        self.outputs.push(output);
    }

    pub fn add_input(&mut self, input: usize) {
        self.inputs.push(input);
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ID: {}\nOperation: {:#}\nInputs: {:#?}\nOutputs: {:#?}",
            self.id, self.operation, self.inputs, self.outputs,
        )
    }
}
