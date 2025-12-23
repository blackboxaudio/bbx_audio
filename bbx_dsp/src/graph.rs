use std::collections::HashMap;

use crate::{
    block::{BlockId, BlockType},
    blocks::{
        effectors::overdrive::OverdriveBlock,
        generators::oscillator::OscillatorBlock,
        io::{file_input::FileInputBlock, file_output::FileOutputBlock, output::OutputBlock},
        modulators::lfo::LfoBlock,
    },
    buffer::{AudioBuffer, Buffer},
    context::DspContext,
    parameter::Parameter,
    reader::Reader,
    sample::Sample,
    waveform::Waveform,
    writer::Writer,
};

/// Used for storing information about which blocks are connected
/// and in what way.
#[derive(Debug, Clone)]
pub struct Connection {
    pub from: BlockId,
    pub from_output: usize,
    pub to: BlockId,
    pub to_input: usize,
}

/// Used for storing all relevant data about a DSP `Graph`,
/// including its blocks, `AudioBuffer`s and modulation values for each block,
/// what order to execute calculations in, and so forth.
pub struct Graph<S: Sample> {
    blocks: Vec<BlockType<S>>,
    connections: Vec<Connection>,
    execution_order: Vec<BlockId>,
    output_block: Option<BlockId>,

    // Pre-allocated buffers
    audio_buffers: Vec<AudioBuffer<S>>,
    modulation_values: Vec<S>,

    // Buffer management
    block_buffer_start: Vec<usize>,
    buffer_size: usize,
    context: DspContext,
}

impl<S: Sample> Graph<S> {
    /// Create a `Graph` with a given sample rate, buffer size, and number of channels.
    pub fn new(sample_rate: f64, buffer_size: usize, num_channels: usize) -> Self {
        let context = DspContext {
            sample_rate,
            buffer_size,
            num_channels,
            current_sample: 0,
        };

        Self {
            blocks: Vec::new(),
            connections: Vec::new(),
            execution_order: Vec::new(),
            output_block: None,
            audio_buffers: Vec::new(),
            modulation_values: Vec::new(),
            block_buffer_start: Vec::new(),
            buffer_size,
            context,
        }
    }

    /// Get the underlying `DspContext` used by a `Graph`.
    #[inline]
    pub fn context(&self) -> &DspContext {
        &self.context
    }

    /// Add an arbitrary block to the `Graph`.
    pub fn add_block(&mut self, block: BlockType<S>) -> BlockId {
        let block_id = BlockId(self.blocks.len());

        self.block_buffer_start.push(self.audio_buffers.len());
        self.blocks.push(block);

        let output_count = self.blocks[block_id.0].output_count();
        for _ in 0..output_count {
            self.audio_buffers.push(AudioBuffer::new(self.buffer_size));
        }

        block_id
    }

    /// Add an output block to the `Graph`.
    pub fn add_output_block(&mut self) -> BlockId {
        let block = BlockType::Output(OutputBlock::<S>::new(self.context.num_channels));
        let block_id = self.add_block(block);
        self.output_block = Some(block_id);
        block_id
    }

    /// Form a `Connection` between two particular blocks.
    pub fn connect(&mut self, from: BlockId, from_output: usize, to: BlockId, to_input: usize) {
        self.connections.push(Connection {
            from,
            from_output,
            to,
            to_input,
        })
    }

    /// Prepares the `Graph` to be processed.
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

    /// Process the buffers for each of the `Graph`'s blocks.
    pub fn process_buffers(&mut self, output_buffers: &mut [&mut [S]]) {
        // Clear all buffers
        for buffer in &mut self.audio_buffers {
            buffer.zeroize();
        }

        // Process blocks according to execution order
        for i in 0..self.execution_order.len() {
            let block_id = self.execution_order[i];
            self.process_block_unsafe(block_id);
            self.collect_modulation_values(block_id);
        }

        // Copy final output to the provided buffer
        self.copy_to_output_buffer(output_buffers);
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
        // 1. Input indices come from other blocks' outputs.
        // 2. Output indices are unique to this block.
        // 3. Therefore, input_indices and output_indices NEVER overlap.
        // 4. All indices are valid (within the bounds of self.audio_buffers).
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
                .map(|index| {
                    let buffer_ptr = buffers_ptr.add(index);
                    std::slice::from_raw_parts_mut((&mut *buffer_ptr).as_mut_ptr(), (&*buffer_ptr).len())
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
        if let Some(output_block_id) = self.output_block {
            let output_count = self.blocks[output_block_id.0].output_count();
            for channel in 0..output_count.min(output_buffer.len()) {
                let internal_buffer_index = self.get_buffer_index(output_block_id, channel);
                let internal_buffer = &self.audio_buffers[internal_buffer_index];

                let copy_length = internal_buffer.len().min(output_buffer[channel].len());
                output_buffer[channel][..copy_length].copy_from_slice(&internal_buffer.as_slice()[..copy_length]);
            }
        }
    }

    #[inline]
    fn get_buffer_index(&self, block_id: BlockId, output_index: usize) -> usize {
        self.block_buffer_start[block_id.0] + output_index
    }

    // =========================================================================
    // Configuration and FFI support methods
    // =========================================================================

    /// Create a graph from a JSON configuration string.
    pub fn from_config(
        config_json: &str,
        sample_rate: f64,
        buffer_size: usize,
        num_channels: usize,
    ) -> Result<Self, crate::config::ConfigError> {
        let config = crate::config::GraphConfig::from_json(config_json)?;
        config.build_graph(sample_rate, buffer_size, num_channels)
    }

    /// Set a modulation connection between a modulator block and a target parameter.
    pub fn set_modulation(
        &mut self,
        source_id: BlockId,
        target_id: BlockId,
        param_name: &str,
        _depth: S,
    ) -> Result<(), crate::config::ConfigError> {
        if target_id.0 >= self.blocks.len() {
            return Err(crate::config::ConfigError::InvalidConnection(
                format!("Target block {} does not exist", target_id.0),
            ));
        }

        self.blocks[target_id.0]
            .set_parameter(param_name, Parameter::Modulated(source_id))
            .map_err(|e| crate::config::ConfigError::InvalidParameter(e))
    }

    /// Set a parameter value on a block.
    pub fn set_parameter(&mut self, block_id: BlockId, param_name: &str, value: S) {
        if block_id.0 < self.blocks.len() {
            let _ = self.blocks[block_id.0].set_parameter(param_name, Parameter::Constant(value));
        }
    }

    /// Bind a parameter to an external atomic source (for JUCE integration).
    ///
    /// # Safety
    /// The provided pointer must remain valid for the lifetime of the binding.
    pub fn bind_external_parameter(
        &mut self,
        block_id: BlockId,
        param_name: &str,
        atomic_ptr: *const crate::parameter::AtomicF32,
    ) {
        if block_id.0 < self.blocks.len() {
            let _ = self.blocks[block_id.0].set_parameter(param_name, Parameter::External(atomic_ptr));
        }
    }

    /// Reset all DSP state in the graph.
    pub fn reset(&mut self) {
        // Reset all block states
        for block in &mut self.blocks {
            match block {
                BlockType::Filter(f) => f.reset(),
                BlockType::DcBlocker(d) => d.reset(),
                _ => {}
            }
        }

        // Clear all buffers
        for buffer in &mut self.audio_buffers {
            buffer.zeroize();
        }

        // Reset modulation values
        for value in &mut self.modulation_values {
            *value = S::ZERO;
        }
    }

    /// Get the number of blocks in the graph.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Get mutable access to a block by ID.
    pub fn get_block_mut(&mut self, block_id: BlockId) -> Option<&mut BlockType<S>> {
        self.blocks.get_mut(block_id.0)
    }
}

/// Used for easily constructing a DSP `Graph`.
pub struct GraphBuilder<S: Sample> {
    graph: Graph<S>,
}

impl<S: Sample> GraphBuilder<S> {
    /// Create a `GraphBuilder` that will construct a `Graph` with a given
    /// sample rate, buffer size, and number of channels.
    pub fn new(sample_rate: f64, buffer_size: usize, num_channels: usize) -> Self {
        Self {
            graph: Graph::new(sample_rate, buffer_size, num_channels),
        }
    }

    // I/O

    /// Add a `FileInputBlock` to the `Graph`, which is useful for processing
    /// an audio file with the rest of the DSP `Graph`.
    pub fn add_file_input(&mut self, reader: Box<dyn Reader<S>>) -> BlockId {
        let block = BlockType::FileInput(FileInputBlock::new(reader));
        self.graph.add_block(block)
    }

    /// Add a `FileOutputBlock` to the `Graph`, which is useful for rendering
    /// an audio file of the DSP `Graph`.
    pub fn add_file_output(&mut self, writer: Box<dyn Writer<S>>) -> BlockId {
        let block = BlockType::FileOutput(FileOutputBlock::new(writer));
        self.graph.add_block(block)
    }

    // GENERATORS

    /// Add an `OscillatorBlock` to the `Graph`.
    pub fn add_oscillator(&mut self, frequency: f64, waveform: Waveform, seed: Option<u64>) -> BlockId {
        let block = BlockType::Oscillator(OscillatorBlock::new(S::from_f64(frequency), waveform, seed));
        self.graph.add_block(block)
    }

    // EFFECTORS

    /// Add an `OverdriveBlock` to the `Graph`.
    pub fn add_overdrive(&mut self, drive: f64, level: f64, tone: f64, sample_rate: f64) -> BlockId {
        let block = BlockType::Overdrive(OverdriveBlock::new(
            S::from_f64(drive),
            S::from_f64(level),
            tone,
            sample_rate,
        ));
        self.graph.add_block(block)
    }

    // MODULATORS

    /// Add an `LfoBlock` to the `Graph`, which is useful when wanting to
    /// modulate one or more `Parameter`s of one or more blocks.
    pub fn add_lfo(&mut self, frequency: f64, depth: f64, seed: Option<u64>) -> BlockId {
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
            seed,
        ));
        self.graph.add_block(block)
    }

    /// Form a `Connection` between two particular blocks.
    pub fn connect(&mut self, from: BlockId, from_output: usize, to: BlockId, to_input: usize) -> &mut Self {
        self.graph.connect(from, from_output, to, to_input);
        self
    }

    /// Specify a `Parameter` to be modulated by a `Modulator` block.
    pub fn modulate(&mut self, source: BlockId, target: BlockId, parameter: &str) -> &mut Self {
        if let Err(e) = self.graph.blocks[target.0].set_parameter(parameter, Parameter::Modulated(source)) {
            eprintln!("Modulation error: {e}");
        }
        self
    }

    /// Prepare the final DSP `Graph`.
    pub fn build(mut self) -> Graph<S> {
        // TODO: Fix this logic to work with ALL last blocks that do not yet have an output
        // Currently this logic would make so that if multiple oscillators are used, only
        // one of them would be connected to the output.
        if let Some(last_block) = self.graph.topological_sort().last() {
            let output = self.graph.add_output_block();
            for channel_index in 0..self.graph.context.num_channels {
                self.connect(*last_block, 0, output, channel_index);
            }
        }

        self.graph.prepare_for_playback();
        self.graph
    }
}
