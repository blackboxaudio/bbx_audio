use std::fmt::{Display, Formatter};

use rand::Rng;

use crate::process::Process;

pub type Operation = Box<dyn Process + Send>;

/// The representation of a DSP operation within a `Graph`.
pub struct Block {
    pub id: usize,

    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,

    pub operation: Operation,
}

impl Block {
    pub fn new(operation: Operation) -> Block {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<usize>();
        return Block {
            id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            operation,
        };
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
