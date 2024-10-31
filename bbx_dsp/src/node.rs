use std::collections::HashMap;

use rand::Rng;

use crate::{
    context::Context,
    effector::Effector,
    generator::Generator,
    modulator::{ModulationDestination, Modulator},
    operation::{Operation, OperationType},
};

/// The identifier for a node in a graph.
pub type NodeId = usize;

/// Represents an implementation that generates audio output buffers from a number of audio inputs.
pub struct Node {
    /// The identifier of a `Node`.
    pub id: NodeId,

    /// The context in which the graph is being evaluated.
    pub context: Context,

    /// The `NodeId`s of the incoming nodes.
    pub inputs: Vec<NodeId>,

    /// The `NodeId`s of the output nodes.
    pub outputs: Vec<NodeId>,

    pub modulations: HashMap<NodeId, ModulationDestination>,

    /// The associated `Operation` for this `Node`.
    pub operation: Operation,

    /// The associated `OperationType` for this `Node`.
    pub operation_type: OperationType,
}

impl Node {
    pub fn new(context: Context, operation: Operation, operation_type: OperationType) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<NodeId>();
        Node {
            id,
            context,
            inputs: Vec::with_capacity(context.max_num_graph_nodes),
            outputs: Vec::with_capacity(context.max_num_graph_nodes),
            modulations: HashMap::with_capacity(context.max_num_graph_nodes),
            operation,
            operation_type,
        }
    }

    pub fn from_effector(context: Context, effector: Effector) -> Self {
        Self::new(context, effector.to_operation(context), OperationType::Effector)
    }

    pub fn from_generator(context: Context, generator: Generator) -> Self {
        Self::new(context, generator.to_operation(context), OperationType::Generator)
    }

    pub fn from_modulator(context: Context, modulator: Modulator) -> Self {
        Self::new(context, modulator.to_operation(context), OperationType::Modulator)
    }
}

impl Node {
    pub fn add_output(&mut self, output: NodeId) {
        self.outputs.push(output);
    }

    pub fn add_input(&mut self, input: NodeId) {
        self.inputs.push(input);
    }

    pub fn add_modulation(&mut self, mod_id: NodeId, mod_destination: ModulationDestination) {
        self.modulations.insert(mod_id, mod_destination);
    }
}
