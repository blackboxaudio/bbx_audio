use std::collections::HashMap;

use crate::{
    block::{BlockId, BlockType},
    blocks::{generators::oscillator::OscillatorBlock, modulators::lfo::LfoBlock, output::OutputBlock},
    context::DspContext,
    parameter::Parameter,
    sample::Sample,
    waveform::Waveform,
};

#[derive(Debug, Clone)]
pub struct Connection {
    pub from: BlockId,
    pub from_output: usize,
    pub to: BlockId,
    pub to_input: usize,
}

pub struct Graph<S: Sample> {
    blocks: Vec<BlockType<S>>,
    connections: Vec<Connection>,
    execution_order: Vec<BlockId>,
    output_blocks: Vec<BlockId>,

    // Pre-allocated buffers
    audio_buffers: Vec<Vec<S>>,
    modulation_values: Vec<S>,

    // Buffer management
    block_buffer_start: Vec<usize>,
    buffer_size: usize,
    channels: usize,
    context: DspContext,
}

impl<S: Sample> Graph<S> {
    pub fn new(sample_rate: f64, buffer_size: usize, channels: usize) -> Self {
        let context = DspContext {
            sample_rate,
            buffer_size,
            channels,
            current_sample: 0,
        };

        Self {
            blocks: Vec::new(),
            connections: Vec::new(),
            execution_order: Vec::new(),
            output_blocks: Vec::new(),
            audio_buffers: Vec::new(),
            modulation_values: Vec::new(),
            block_buffer_start: Vec::new(),
            buffer_size,
            channels,
            context,
        }
    }

    pub fn context(&self) -> &DspContext {
        &self.context
    }

    pub fn add_block(&mut self, block: BlockType<S>) -> BlockId {
        let block_id = BlockId(self.blocks.len());

        self.block_buffer_start.push(self.audio_buffers.len());
        self.blocks.push(block);

        let output_count = self.blocks[block_id.0].output_count();
        for _ in 0..output_count {
            self.audio_buffers.push(vec![S::ZERO; self.buffer_size * self.channels]);
        }

        block_id
    }

    pub fn add_output_block(&mut self, channels: usize) -> BlockId {
        let block = BlockType::Output(OutputBlock::<S>::new(channels));
        let block_id = self.add_block(block);
        self.output_blocks.push(block_id);
        block_id
    }

    pub fn connect(&mut self, from: BlockId, from_output: usize, to: BlockId, to_input: usize) {
        self.connections.push(Connection {
            from,
            from_output,
            to,
            to_input,
        })
    }

    pub fn prepare_for_playback(&mut self) {
        self.execution_order = self.topological_sort();
        self.modulation_values.resize(self.blocks.len(), S::ZERO);
    }

    fn topological_sort(&self) -> Vec<BlockId> {
        let mut in_degree = vec![0; self.blocks.len()];
        let mut adjacency_list: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

        // Build adjacency list and calculate in-degrees
        for connection in &self.connections {
            adjacency_list.entry(connection.from).or_default().push(connection.to);
            in_degree[connection.to.0] += 1;
        }

        // Kahn's algorithm
        let mut queue = Vec::new();
        let mut result = Vec::new();

        for (i, &degree) in in_degree.iter().enumerate() {
            if degree == 0 {
                queue.push(BlockId(i));
            }
        }

        while let Some(block) = queue.pop() {
            result.push(block);
            if let Some(neighbors) = adjacency_list.get(&block) {
                for &neighbor in neighbors {
                    in_degree[neighbor.0] -= 1;
                    if in_degree[neighbor.0] == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }

        result
    }

    pub fn process_buffer(&mut self, output_buffer: &mut [&mut [S]]) {
        // Clear all buffers
        for buffer in &mut self.audio_buffers {
            buffer.fill(S::ZERO);
        }

        // Process blocks according to execution order
        for i in 0..self.execution_order.len() {
            let block_id = self.execution_order[i];
            self.process_block_unsafe(block_id);
            self.collect_modulation_values(block_id);
        }

        // Copy final output to the provided buffer
        self.copy_to_output_buffer(output_buffer);
    }

    fn process_block_unsafe(&mut self, block_id: BlockId) {
        let mut input_indices = Vec::new();
        let mut output_indices = Vec::new();

        for connection in &self.connections {
            if connection.to == block_id {
                let buffer_index = self.get_buffer_index(connection.from, connection.from_output);
                input_indices.push(buffer_index);
            }
        }

        let output_count = self.blocks[block_id.0].output_count();
        for output_index in 0..output_count {
            let buffer_index = self.get_buffer_index(block_id, output_index);
            output_indices.push(buffer_index);
        }

        // SAFETY: Our buffer indexing guarantees that:
        // 1. Input indices come from other blocks' outputs
        // 2. Output indices are unique to this block
        // 3. Therefore, input_indices and output_indices NEVER overlap
        // 4. All indices are valid (within the bounds of self.audio_buffers)
        unsafe {
            let buffers_ptr = self.audio_buffers.as_mut_ptr();

            let input_slices: Vec<&[S]> = input_indices
                .into_iter()
                .map(|index| {
                    let buffer_ptr = buffers_ptr.add(index);
                    std::slice::from_raw_parts((&*buffer_ptr).as_ptr(), (&*buffer_ptr).len())
                })
                .collect();
            let mut output_slices: Vec<&mut [S]> = output_indices
                .into_iter()
                .map(|idx| {
                    let buffer_ptr = buffers_ptr.add(idx);
                    std::slice::from_raw_parts_mut((&mut *buffer_ptr).as_mut_ptr(), (&mut *buffer_ptr).len())
                })
                .collect();

            self.blocks[block_id.0].process(
                &input_slices,
                &mut output_slices,
                &self.modulation_values,
                &self.context,
            );
        }
    }

    fn collect_modulation_values(&mut self, block_id: BlockId) {
        let has_modulation = !self.blocks[block_id.0].modulation_outputs().is_empty();
        if has_modulation {
            let buffer_index = self.get_buffer_index(block_id, 0);
            if !self.audio_buffers[buffer_index].is_empty() {
                self.modulation_values[block_id.0] = self.audio_buffers[buffer_index][0];
            }
        }
    }

    fn copy_to_output_buffer(&self, output_buffer: &mut [&mut [S]]) {
        // In a more complex system, there could be multiple output blocks...
        if let Some(&output_block_id) = self.output_blocks.first() {
            let output_count = self.blocks[output_block_id.0].output_count();
            for channel in 0..output_count.min(output_buffer.len()) {
                let internal_buffer_index = self.get_buffer_index(output_block_id, channel);
                let internal_buffer = &self.audio_buffers[internal_buffer_index];

                let copy_length = internal_buffer.len().min(output_buffer[channel].len());
                output_buffer[channel][..copy_length].copy_from_slice(&internal_buffer[..copy_length]);
            }
        }
    }

    fn get_buffer_index(&self, block_id: BlockId, output_index: usize) -> usize {
        self.block_buffer_start[block_id.0] + output_index
    }
}

// GRAPH BUILDER

pub struct GraphBuilder<S: Sample> {
    graph: Graph<S>,
}

impl<S: Sample> GraphBuilder<S> {
    pub fn new(sample_rate: f64, buffer_size: usize, channels: usize) -> Self {
        Self {
            graph: Graph::new(sample_rate, buffer_size, channels),
        }
    }

    pub fn add_output(&mut self, channels: usize) -> BlockId {
        self.graph.add_output_block(channels)
    }

    pub fn add_oscillator(&mut self, frequency: f64, waveform: Waveform) -> BlockId {
        let block = BlockType::Oscillator(OscillatorBlock::new(S::from_f64(frequency), waveform));
        self.graph.add_block(block)
    }

    pub fn add_lfo(&mut self, frequency: f64, depth: f64) -> BlockId {
        // Because the modulation is happening at *control rate*, we are
        // limited to a frequency that is 1/2 of the sample rate divided
        // by the buffer size. Audio rate modulation is not supported because:
        // 1. Processing modulation at audio rate is too CPU-intensive.
        // 2. Most musical modulation happens below 20Hz.
        // 3. Control rate limitations help avoid artifacts from aliasing.
        let max_frequency = 0.5 * (self.graph.context.sample_rate / self.graph.context.buffer_size as f64);
        let clamped_frequency = frequency.clamp(0.01, max_frequency);

        let block = BlockType::Lfo(LfoBlock::new(
            S::from_f64(clamped_frequency),
            S::from_f64(depth),
            Waveform::Sine,
        ));
        self.graph.add_block(block)
    }

    pub fn connect(&mut self, from: BlockId, from_output: usize, to: BlockId, to_input: usize) -> &mut Self {
        self.graph.connect(from, from_output, to, to_input);
        self
    }

    pub fn modulate(&mut self, source: BlockId, target: BlockId, parameter: &str) -> &mut Self {
        if let Err(e) = self.graph.blocks[target.0].set_parameter(parameter, Parameter::Modulated(source)) {
            eprintln!("Modulation error: {}", e);
        }
        self
    }

    pub fn build(mut self) -> Graph<S> {
        self.graph.prepare_for_playback();
        self.graph
    }
}
