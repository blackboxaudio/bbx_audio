use rand::Rng;

use crate::{
    context::Context,
    generator::Generator,
    node::NodeId,
    operation::{Operation, OperationType},
};
use crate::effector::Effector;

/// Represents an implementation that generates audio output buffers from a number of audio inputs.
pub struct Block {
    /// The identifier of a `Block`.
    pub id: NodeId,

    /// The context in which the graph is being evaluated.
    pub context: Context,

    /// The `NodeId`s of the incoming nodes.
    pub inputs: Vec<NodeId>,

    /// The `NodeId`s of the output nodes.
    pub outputs: Vec<NodeId>,

    /// The associated `Operation` for this `Block`.
    pub operation: Operation,

    /// The associated `OperationType` for this `Block`.
    pub operation_type: OperationType,
}

impl Block {
    fn new(context: Context, operation: Operation, operation_type: OperationType) -> Block {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<NodeId>();
        Block {
            id,
            context,
            inputs: Vec::with_capacity(context.max_num_graph_nodes),
            outputs: Vec::with_capacity(context.max_num_graph_nodes),
            operation,
            operation_type,
        }
    }

    pub fn from_effector(context: Context, effector: Effector) -> Block {
        Self::new(context, effector.to_operation(context), OperationType::Effector)
    }

    pub fn from_generator(context: Context, generator: Generator) -> Block {
        Self::new(context, generator.to_operation(context), OperationType::Generator)
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
