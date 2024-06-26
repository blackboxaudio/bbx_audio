use crate::{block::Block, process::Process, sample::Sample};

/// A collection of interconnected `Block` objects.
pub struct Graph {
    sample_rate: usize,
    blocks: Vec<Block>,
}

impl Graph {
    pub fn new(sample_rate: usize) -> Graph {
        return Graph {
            sample_rate,
            blocks: vec![],
        };
    }

    pub fn sample_rate(&self) -> usize {
        return self.sample_rate;
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

impl Graph {
    pub fn evaluate(&mut self) -> Sample<f32> {
        let mut sample = 0.0;
        for block in self.blocks.iter_mut() {
            match block {
                Block::Effector(effector) => {
                    sample = effector.process(Some(sample));
                }
                Block::Generator(generator) => {
                    sample = generator.process(None);
                }
            }
        }
        return sample;
    }
}
