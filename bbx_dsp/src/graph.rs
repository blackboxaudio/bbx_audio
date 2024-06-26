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
        self.update_processing_order();
    }

    pub fn get_block(&self, id: usize) -> Option<&Block> {
        return self.blocks.iter().find(|&block| block.id == id);
    }

    fn update_processing_order(&mut self) {
        let mut stack: Vec<usize> = Vec::with_capacity(self.blocks.len());
        let mut visited: Vec<usize> = Vec::with_capacity(self.blocks.len());

        fn dfs(block: &Block, order: &mut Vec<usize>, visited: &mut Vec<usize>, blocks: &Vec<Block>) {
            visited.push(block.id);
            for &block_id in &block.outputs {
                if visited.contains(&block_id) {
                    continue
                } else {
                    let block_option = blocks.iter().find(|&block| block.id == block_id);
                    if let Some(block) = block_option {
                        dfs(block, order, visited, blocks);
                    }
                }
            }
            order.push(block.id);
        }

        for block in &self.blocks {
            if visited.contains(&block.id) {
                continue
            } else {
                dfs(block, &mut stack, &mut visited, &self.blocks);
            }
        }

        if stack.len() == self.blocks.len() {
            // TODO: Fix checks for: cycles, disconnected graphs
            println!("GOOD\n{:#?}", stack);
        } else {
            println!("BAD\n{:#?}", stack);
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
