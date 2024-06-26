use crate::{block::Block, sample::Sample};

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
}

impl Graph {
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
        self.sort_blocks();
    }

    fn sort_blocks(&mut self) {
        let mut in_degrees = vec![0; self.blocks.len()];
        for block in &self.blocks {
            for &output in &block.outputs {
                in_degrees[output] += 1;
            }
        }

        let mut queue = std::collections::VecDeque::new();
        for (id, &degree) in in_degrees.iter().enumerate() {
            if degree == 0 {
                queue.push_back(id)
            }
        }

        let mut order = Vec::new();
        while let Some(block_id) = queue.pop_front() {
            order.push(block_id);
            for &output in &self.blocks[block_id].outputs {
                in_degrees[output] -= 1;
                if in_degrees[output] == 0 {
                    queue.push_back(output);
                }
            }
        }

        if order.len() == self.blocks.len() {
            println!("{:#?}", order);
        }
    }
}

impl Graph {
    pub fn evaluate(&mut self) -> Sample {
        let mut sample = 0.0;
        for block in self.blocks.iter_mut() {
            sample = block.operation.process(Some(sample));
        }
        return sample;
    }
}
