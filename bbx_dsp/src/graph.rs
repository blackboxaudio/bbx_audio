//! DSP graph system.
//!
//! This module provides [`Graph`] for managing connected DSP blocks and
//! [`GraphBuilder`] for fluent graph construction.
//!
//! Blocks are connected to form a signal processing chain. The graph handles
//! buffer allocation, execution ordering via topological sort, and modulation
//! value collection.

use alloc::{string::String, vec, vec::Vec};
use core::fmt;
use std::collections::HashMap;

use bbx_core::{Buffer, StackVec};

// Re-export for backwards compatibility
pub use crate::block::{MAX_BLOCK_INPUTS, MAX_BLOCK_OUTPUTS};
use crate::{
    block::{BlockCategory, BlockId, BlockType},
    blocks::{effectors::mixer::MixerBlock, io::output::OutputBlock},
    buffer::SampleBuffer,
    channel::ChannelLayout,
    context::DspContext,
    parameter::{ModulationRoute, ModulationSource, ModulationValues, ParameterError},
    sample::Sample,
};

/// Describes an audio connection between two blocks.
///
/// Connects a specific output port of one block to an input port of another.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Source block providing audio.
    pub from: BlockId,
    /// Output port index on the source block.
    pub from_output: usize,
    /// Destination block receiving audio.
    pub to: BlockId,
    /// Input port index on the destination block.
    pub to_input: usize,
}

/// Snapshot of a block's metadata for visualization.
///
/// Contains owned data suitable for cross-thread transfer.
#[derive(Debug, Clone)]
pub struct BlockSnapshot {
    /// The block's unique identifier.
    pub id: usize,
    /// Display name of the block type.
    pub name: String,
    /// Category of the block.
    pub category: BlockCategory,
    /// Number of input ports.
    pub input_count: usize,
    /// Number of output ports.
    pub output_count: usize,
}

/// Snapshot of a connection for visualization.
#[derive(Debug, Clone)]
pub struct ConnectionSnapshot {
    /// Source block ID.
    pub from_block: usize,
    /// Source output port index.
    pub from_output: usize,
    /// Destination block ID.
    pub to_block: usize,
    /// Destination input port index.
    pub to_input: usize,
}

/// Snapshot of a modulation connection for visualization.
#[derive(Debug, Clone)]
pub struct ModulationConnectionSnapshot {
    /// Source modulator block ID.
    pub from_block: usize,
    /// Index of the source block's modulation output.
    pub from_output: usize,
    /// Target block ID.
    pub to_block: usize,
    /// Name of the modulated parameter on the target block.
    pub parameter_name: String,
    /// Scale applied to the source value.
    pub depth: f64,
}

/// Snapshot of a graph's topology for visualization.
///
/// Contains all block metadata and connections at a point in time.
/// This is an owned snapshot suitable for cross-thread transfer.
#[derive(Debug, Clone)]
pub struct GraphTopologySnapshot {
    /// All blocks in the graph.
    pub blocks: Vec<BlockSnapshot>,
    /// All audio connections between blocks.
    pub connections: Vec<ConnectionSnapshot>,
    /// All modulation connections from modulators to block parameters.
    pub modulation_connections: Vec<ModulationConnectionSnapshot>,
}

/// Errors raised while wiring a graph.
///
/// These surface at build time, never on the audio thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// The referenced block does not exist in this graph.
    BlockNotFound(BlockId),
    /// The target block has no parameter with this name.
    UnknownParameter {
        /// The target block.
        block: BlockId,
        /// The name that was requested.
        name: String,
    },
    /// The source block has fewer modulation outputs than the requested index.
    InvalidModulationOutput {
        /// The source block.
        block: BlockId,
        /// The requested output index.
        output: usize,
        /// How many modulation outputs the block declares.
        available: usize,
    },
    /// The parameter refused the route.
    Parameter {
        /// The target block.
        block: BlockId,
        /// The parameter name.
        name: String,
        /// The underlying parameter error.
        error: ParameterError,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockNotFound(block) => write!(formatter, "block {} does not exist", block.0),
            Self::UnknownParameter { block, name } => {
                write!(formatter, "block {} has no parameter named \"{name}\"", block.0)
            }
            Self::InvalidModulationOutput {
                block,
                output,
                available,
            } => write!(
                formatter,
                "block {} has {available} modulation output(s); output {output} does not exist",
                block.0
            ),
            Self::Parameter { block, name, error } => {
                write!(formatter, "parameter \"{name}\" on block {}: {error}", block.0)
            }
        }
    }
}

impl std::error::Error for GraphError {}

/// A directed acyclic graph of connected DSP blocks.
///
/// The graph manages block storage, buffer allocation, and execution ordering.
/// Blocks are processed in topologically sorted order to ensure dependencies
/// are satisfied.
pub struct Graph<S: Sample> {
    blocks: Vec<BlockType<S>>,
    connections: Vec<Connection>,
    execution_order: Vec<BlockId>,
    output_block: Option<BlockId>,

    // Pre-allocated buffers
    audio_buffers: Vec<SampleBuffer<S>>,
    modulation_values: Vec<S>,
    modulation_offsets: Vec<usize>,

    // Buffer management
    block_buffer_start: Vec<usize>,
    buffer_size: usize,
    context: DspContext,

    // Pre-computed connection lookups: block_id -> [buffer index per input port].
    // Indexed by port so a block sees its inputs at the ports it declared;
    // an unconnected port below the highest connected one is None (empty slice).
    block_input_buffers: Vec<Vec<Option<usize>>>,
}

impl<S: Sample> Graph<S> {
    /// Create a `Graph` with a given sample rate, buffer size, and number of channels.
    pub fn new(sample_rate: f64, buffer_size: usize, num_channels: usize) -> Self {
        let context = DspContext {
            sample_rate,
            buffer_size,
            num_channels,
            current_sample: 0,
            channel_layout: ChannelLayout::default(),
        };

        Self {
            blocks: Vec::new(),
            connections: Vec::new(),
            execution_order: Vec::new(),
            output_block: None,
            audio_buffers: Vec::new(),
            modulation_values: Vec::new(),
            modulation_offsets: Vec::new(),
            block_buffer_start: Vec::new(),
            buffer_size,
            context,
            block_input_buffers: Vec::new(),
        }
    }

    /// Get the underlying `DspContext` used by a `Graph`.
    #[inline]
    pub fn context(&self) -> &DspContext {
        &self.context
    }

    /// The modulation values collected during the most recent buffer.
    ///
    /// Useful for inspecting what a [`Parameter`](crate::parameter::Parameter)
    /// resolved to, for example in tests or a debug view.
    #[inline]
    pub fn modulation_values(&self) -> ModulationValues<'_, S> {
        ModulationValues::new(&self.modulation_values, &self.modulation_offsets)
    }

    /// Get a reference to a block by its ID.
    #[inline]
    pub fn get_block(&self, id: BlockId) -> Option<&BlockType<S>> {
        self.blocks.get(id.0)
    }

    /// Get a mutable reference to a block by its ID.
    #[inline]
    pub fn get_block_mut(&mut self, id: BlockId) -> Option<&mut BlockType<S>> {
        self.blocks.get_mut(id.0)
    }

    /// Prepare the graph for processing with audio context parameters.
    ///
    /// Call this when the sample rate, buffer size, or channel count changes.
    /// Propagates to all blocks, allowing them to recalculate coefficients
    /// and reset state that would cause glitches at the new settings.
    ///
    /// This method also computes the execution order and pre-allocates buffers.
    /// It is called automatically by [`GraphBuilder::build()`].
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate in Hz
    /// * `buffer_size` - Buffer size in samples
    /// * `num_channels` - Number of audio channels
    pub fn prepare(&mut self, sample_rate: f64, buffer_size: usize, num_channels: usize) {
        self.context.sample_rate = sample_rate;
        self.context.buffer_size = buffer_size;
        self.context.num_channels = num_channels;
        self.buffer_size = buffer_size;

        for block in &mut self.blocks {
            block.prepare(&self.context);
        }

        self.execution_order = self.topological_sort();

        // A prefix-sum table lets a ModulationSource resolve to a flat index
        // (offsets[block] + output) with no search on the audio thread
        let mut end = 0;
        self.modulation_offsets.clear();
        self.modulation_offsets.push(end);
        for block in &self.blocks {
            end += block.modulation_outputs().len();
            self.modulation_offsets.push(end);
        }
        self.modulation_values.clear();
        self.modulation_values.resize(end, S::ZERO);

        // Buffers were sized when their block was added; a host may prepare
        // again with a different buffer size, so bring every buffer to it
        for buffer in &mut self.audio_buffers {
            if buffer.len() != buffer_size {
                *buffer = SampleBuffer::new(buffer_size);
            }
        }

        // Pre-compute input buffer indices for each block (O(1) lookup during processing)
        self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
        for connection in &self.connections {
            let buffer_index = self.get_buffer_index(connection.from, connection.from_output);
            let ports = &mut self.block_input_buffers[connection.to.0];
            if ports.len() <= connection.to_input {
                ports.resize(connection.to_input + 1, None);
            }
            assert!(
                ports[connection.to_input].is_none(),
                "Block {} input {} is connected twice; sum sources with a MixerBlock instead",
                connection.to.0,
                connection.to_input
            );
            ports[connection.to_input] = Some(buffer_index);
        }

        #[cfg(debug_assertions)]
        self.validate_buffer_indices();
    }

    /// Reset all blocks in the graph to their initial state.
    ///
    /// Clears delay lines, filter states, phase accumulators, etc.
    /// Useful when starting fresh playback or when the audio stream
    /// is discontinuous.
    pub fn reset(&mut self) {
        for block in &mut self.blocks {
            block.reset();
        }
    }

    /// Add an arbitrary block to the `Graph`.
    pub fn add_block(&mut self, block: BlockType<S>) -> BlockId {
        let block_id = BlockId(self.blocks.len());

        self.block_buffer_start.push(self.audio_buffers.len());
        self.blocks.push(block);

        let output_count = self.blocks[block_id.0].output_count();
        for _ in 0..output_count {
            self.audio_buffers.push(SampleBuffer::new(self.buffer_size));
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

    /// Validates that input and output buffer indices never overlap for any block.
    ///
    /// This invariant is critical for the safety of `process_block_unsafe()`.
    #[cfg(debug_assertions)]
    fn validate_buffer_indices(&self) {
        for block_id in 0..self.blocks.len() {
            let input_indices = &self.block_input_buffers[block_id];

            // Compute output indices for this block
            let output_count = self.blocks[block_id].output_count();
            let output_start = self.block_buffer_start[block_id];

            for output_idx in 0..output_count {
                let buffer_idx = output_start + output_idx;

                // Check that no input index matches this output index
                debug_assert!(
                    !input_indices.contains(&Some(buffer_idx)),
                    "Block {block_id} has overlapping input/output buffer index {buffer_idx}. \
                     This would cause undefined behavior in process_block_unsafe()."
                );
            }
        }
    }

    fn topological_sort(&self) -> Vec<BlockId> {
        let mut in_degree = vec![0; self.blocks.len()];
        let mut adjacency_list: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

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

    /// Process one buffer's worth of audio through all blocks.
    ///
    /// Executes blocks in topologically sorted order, copying final output
    /// to the provided buffers (one per channel).
    #[inline]
    pub fn process_buffers(&mut self, output_buffers: &mut [&mut [S]]) {
        for buffer in &mut self.audio_buffers {
            buffer.zeroize();
        }

        for i in 0..self.execution_order.len() {
            let block_id = self.execution_order[i];
            self.process_block_unsafe(block_id);
            self.collect_modulation_values(block_id);
        }

        self.copy_to_output_buffer(output_buffers);
    }

    #[inline]
    fn process_block_unsafe(&mut self, block_id: BlockId) {
        // Use pre-computed input buffer indices (O(1) lookup instead of O(n) scan)
        let input_indices = &self.block_input_buffers[block_id.0];

        // Build output indices using stack allocation (no heap allocation)
        let mut output_indices: StackVec<usize, MAX_BLOCK_OUTPUTS> = StackVec::new();
        let output_count = self.blocks[block_id.0].output_count();
        debug_assert!(
            output_count <= MAX_BLOCK_OUTPUTS,
            "Block output count {output_count} exceeds MAX_BLOCK_OUTPUTS {MAX_BLOCK_OUTPUTS}"
        );
        for output_index in 0..output_count {
            let buffer_index = self.get_buffer_index(block_id, output_index);
            output_indices.push_unchecked(buffer_index);
        }

        // SAFETY: Our buffer indexing guarantees that:
        // 1. Input indices come from other blocks' outputs.
        // 2. Output indices are unique to this block.
        // 3. Therefore, input_indices and output_indices NEVER overlap.
        // 4. All indices are valid (within the bounds of self.audio_buffers).
        unsafe {
            let buffers_ptr = self.audio_buffers.as_mut_ptr();

            // Build input slices using stack allocation (no heap allocation)
            let mut input_slices: StackVec<&[S], MAX_BLOCK_INPUTS> = StackVec::new();
            let input_count = input_indices.len();
            debug_assert!(
                input_count <= MAX_BLOCK_INPUTS,
                "Block input count {input_count} exceeds MAX_BLOCK_INPUTS {MAX_BLOCK_INPUTS}"
            );
            for port in input_indices {
                // An unconnected port reads as an empty slice so the block's
                // own "missing input" default applies
                let slice: &[S] = match port {
                    Some(index) => {
                        let buffer_ptr = buffers_ptr.add(*index);
                        std::slice::from_raw_parts((*buffer_ptr).as_ptr(), (*buffer_ptr).len())
                    }
                    None => &[],
                };
                // SAFETY: We verified input_indices.len() <= MAX_BLOCK_INPUTS via debug_assert
                input_slices.push_unchecked(slice);
            }

            // Build output slices using stack allocation (no heap allocation)
            let mut output_slices: StackVec<&mut [S], MAX_BLOCK_OUTPUTS> = StackVec::new();
            for &index in output_indices.as_slice() {
                let buffer_ptr = buffers_ptr.add(index);
                let slice = std::slice::from_raw_parts_mut((*buffer_ptr).as_mut_ptr(), (*buffer_ptr).len());
                // SAFETY: output_indices.len() <= MAX_BLOCK_OUTPUTS (already verified above)
                output_slices.push_unchecked(slice);
            }

            self.blocks[block_id.0].process(
                input_slices.as_slice(),
                output_slices.as_mut_slice(),
                &ModulationValues::new(&self.modulation_values, &self.modulation_offsets),
                &self.context,
            );
        }
    }

    /// Collect modulation values from modulator blocks.
    ///
    /// # Control-Rate Modulation
    ///
    /// Modulation operates at **control rate** (per-buffer), not audio rate (per-sample).
    /// Only the first sample of each modulator's output is used as the modulation value
    /// for the entire buffer. This has several implications:
    ///
    /// - **LFO frequency limit**: Maximum LFO frequency is `sample_rate / (2 * buffer_size)`. At 44.1kHz with 512
    ///   samples, that's ~43Hz. Higher frequencies will alias.
    /// - **Stepped modulation**: Fast parameter changes appear "stepped" at buffer boundaries.
    /// - **Envelope precision**: Gate on/off detection only happens at buffer boundaries.
    ///
    /// This design is intentional for performance reasons: audio-rate modulation would
    /// require per-sample parameter updates, significantly increasing CPU usage.
    #[inline]
    fn collect_modulation_values(&mut self, block_id: BlockId) {
        let Some(block) = self.blocks.get(block_id.0) else {
            return;
        };
        let output_count = block.modulation_outputs().len();
        let Some(&start) = self.modulation_offsets.get(block_id.0) else {
            return;
        };

        // Modulation output i is read from audio output i; only its first
        // sample is kept because modulation is control rate, not audio rate
        for output in 0..output_count {
            let buffer_index = self.get_buffer_index(block_id, output);
            if let (Some(&first_sample), Some(value)) = (
                self.audio_buffers
                    .get(buffer_index)
                    .and_then(|buffer| buffer.as_slice().first()),
                self.modulation_values.get_mut(start + output),
            ) {
                *value = first_sample;
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
}

/// Fluent builder for constructing DSP graphs.
///
/// Provides methods to add blocks, create connections, and set up modulation.
/// Call [`build`](Self::build) to finalize and prepare the graph.
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

    /// Create a `GraphBuilder` with a specific channel layout.
    ///
    /// This constructor sets both the channel count and the layout, which enables
    /// layout-aware processing for blocks like panners and decoders.
    pub fn with_layout(sample_rate: f64, buffer_size: usize, layout: ChannelLayout) -> Self {
        let num_channels = layout.channel_count();
        let mut builder = Self {
            graph: Graph::new(sample_rate, buffer_size, num_channels),
        };
        builder.graph.context.channel_layout = layout;
        builder
    }

    /// Add a block to the graph.
    ///
    /// Accepts any block type that implements `Into<BlockType<S>>`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use bbx_dsp::prelude::*;
    ///
    /// let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
    /// let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
    /// let gain = builder.add(GainBlock::new(-6.0, None));
    /// builder.connect(osc, 0, gain, 0);
    /// let graph = builder.build();
    /// ```
    pub fn add<B: Into<BlockType<S>>>(&mut self, block: B) -> BlockId {
        self.graph.add_block(block.into())
    }

    /// Form a `Connection` between two particular blocks.
    pub fn connect(&mut self, from: BlockId, from_output: usize, to: BlockId, to_input: usize) -> &mut Self {
        self.graph.connect(from, from_output, to, to_input);
        self
    }

    /// Route `source`'s first modulation output into `parameter` on `target` at unity depth.
    ///
    /// Use [`modulate_with`](Self::modulate_with) to choose another output or a depth.
    pub fn modulate(&mut self, source: BlockId, target: BlockId, parameter: &str) -> Result<&mut Self, GraphError> {
        self.modulate_with(
            ModulationRoute::unity(ModulationSource::first_output(source)),
            target,
            parameter,
        )
    }

    /// Attach `route` to `parameter` on `target`.
    ///
    /// Fails if either block is missing, the source lacks the requested
    /// modulation output, the target has no such parameter, or the parameter
    /// already holds its maximum number of routes.
    pub fn modulate_with(
        &mut self,
        route: ModulationRoute<S>,
        target: BlockId,
        parameter: &str,
    ) -> Result<&mut Self, GraphError> {
        let source = route.source();
        let available = self
            .graph
            .blocks
            .get(source.block.0)
            .ok_or(GraphError::BlockNotFound(source.block))?
            .modulation_outputs()
            .len();
        if source.output >= available {
            return Err(GraphError::InvalidModulationOutput {
                block: source.block,
                output: source.output,
                available,
            });
        }

        let target_block = self
            .graph
            .blocks
            .get_mut(target.0)
            .ok_or(GraphError::BlockNotFound(target))?;
        let slot = target_block
            .parameter_mut(parameter)
            .ok_or_else(|| GraphError::UnknownParameter {
                block: target,
                name: String::from(parameter),
            })?;
        slot.add_route(route).map_err(|error| GraphError::Parameter {
            block: target,
            name: String::from(parameter),
            error,
        })?;
        Ok(self)
    }

    /// Capture a snapshot of the current graph topology for visualization.
    ///
    /// Returns owned data suitable for cross-thread transfer to a visualization
    /// thread. Call this before `build()` to capture the user-defined topology
    /// (the output block is added during build).
    pub fn capture_topology(&self) -> GraphTopologySnapshot {
        let blocks = self
            .graph
            .blocks
            .iter()
            .enumerate()
            .map(|(id, block)| BlockSnapshot {
                id,
                name: block.name().to_string(),
                category: block.category(),
                input_count: block.input_count(),
                output_count: block.output_count(),
            })
            .collect();

        let connections = self
            .graph
            .connections
            .iter()
            .map(|conn| ConnectionSnapshot {
                from_block: conn.from.0,
                from_output: conn.from_output,
                to_block: conn.to.0,
                to_input: conn.to_input,
            })
            .collect();

        let modulation_connections = self
            .graph
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(target_id, block)| {
                block
                    .modulation_routes()
                    .into_iter()
                    .map(move |(parameter_name, route)| ModulationConnectionSnapshot {
                        from_block: route.source().block.0,
                        from_output: route.source().output,
                        to_block: target_id,
                        parameter_name: parameter_name.to_string(),
                        depth: route.depth().to_f64(),
                    })
            })
            .collect();

        GraphTopologySnapshot {
            blocks,
            connections,
            modulation_connections,
        }
    }

    /// Prepare the final DSP `Graph`.
    ///
    /// Automatically inserts a mixer before the output block when multiple terminal
    /// blocks exist, unless the developer has already provided their own mixer or
    /// output block connections.
    ///
    /// # Panics
    ///
    /// Panics if any block has more inputs or outputs than the realtime-safe
    /// limits (`MAX_BLOCK_INPUTS` or `MAX_BLOCK_OUTPUTS`).
    pub fn build(mut self) -> Graph<S> {
        let sample_rate = self.graph.context.sample_rate;
        let buffer_size = self.graph.context.buffer_size;
        let num_channels = self.graph.context.num_channels;

        // Check if developer already added an output block
        let existing_output = self.graph.blocks.iter().position(|b| b.is_output()).map(BlockId);

        // Find all terminal blocks: blocks with no outgoing connections,
        // excluding modulators (LFO, Envelope) and output-type blocks.
        let terminal_blocks: Vec<BlockId> = self
            .graph
            .blocks
            .iter()
            .enumerate()
            .filter(|(idx, block)| {
                let block_id = BlockId(*idx);
                let has_outgoing = self.graph.connections.iter().any(|c| c.from == block_id);
                !has_outgoing && !block.is_modulator() && !block.is_output()
            })
            .map(|(idx, _)| BlockId(idx))
            .collect();

        // Check if developer already added a mixer that receives from terminal blocks
        let explicit_mixer = self.find_explicit_mixer(&terminal_blocks);

        // Determine what needs to be added
        let output_id = existing_output.unwrap_or_else(|| self.graph.add_output_block());

        match (explicit_mixer, terminal_blocks.len()) {
            // Developer provided a mixer - connect it to output if not already connected
            (Some(mixer_id), _) => {
                let mixer_has_outgoing = self.graph.connections.iter().any(|c| c.from == mixer_id);
                if !mixer_has_outgoing {
                    self.connect_block_to_output(mixer_id, output_id, num_channels);
                }
            }

            // No explicit mixer, no terminal blocks - nothing to connect
            (None, 0) => {}

            // No explicit mixer, single terminal block - connect directly to output
            (None, 1) => {
                let block_id = terminal_blocks[0];
                self.connect_block_to_output(block_id, output_id, num_channels);
            }

            // No explicit mixer, multiple terminal blocks - insert a mixer
            (None, num_sources) => {
                let mixer_id = self
                    .graph
                    .add_block(BlockType::Mixer(MixerBlock::new(num_sources, num_channels)));

                // Connect each terminal block's outputs to the mixer's inputs
                for (source_idx, &block_id) in terminal_blocks.iter().enumerate() {
                    let block_output_count = self.graph.blocks[block_id.0].output_count();
                    for ch in 0..num_channels.min(block_output_count) {
                        let mixer_input = source_idx * num_channels + ch;
                        self.connect(block_id, ch, mixer_id, mixer_input);
                    }
                }

                // Connect mixer to output
                self.connect_block_to_output(mixer_id, output_id, num_channels);
            }
        }

        self.graph.prepare(sample_rate, buffer_size, num_channels);

        // Validate that all blocks are within realtime-safe I/O limits
        for (idx, block) in self.graph.blocks.iter().enumerate() {
            let connected_inputs = self.graph.block_input_buffers[idx].len();
            let output_count = block.output_count();

            assert!(
                connected_inputs <= MAX_BLOCK_INPUTS,
                "Block {idx} has {connected_inputs} connected inputs, exceeding MAX_BLOCK_INPUTS ({MAX_BLOCK_INPUTS})"
            );
            assert!(
                output_count <= MAX_BLOCK_OUTPUTS,
                "Block {idx} has {output_count} outputs, exceeding MAX_BLOCK_OUTPUTS ({MAX_BLOCK_OUTPUTS})"
            );
        }

        self.graph
    }

    /// Find an explicit mixer (Mixer or MatrixMixer) that has connections from terminal blocks.
    fn find_explicit_mixer(&self, terminal_blocks: &[BlockId]) -> Option<BlockId> {
        for (idx, block) in self.graph.blocks.iter().enumerate() {
            let is_mixer = matches!(block, BlockType::Mixer(_) | BlockType::MatrixMixer(_));
            if !is_mixer {
                continue;
            }

            let block_id = BlockId(idx);
            let has_terminal_input = self
                .graph
                .connections
                .iter()
                .any(|c| c.to == block_id && terminal_blocks.contains(&c.from));

            if has_terminal_input {
                return Some(block_id);
            }
        }
        None
    }

    /// Connect a block's outputs to the output block, channel by channel.
    fn connect_block_to_output(&mut self, from: BlockId, to: BlockId, num_channels: usize) {
        let output_count = self.graph.blocks[from.0].output_count();
        for ch in 0..num_channels.min(output_count) {
            self.connect(from, ch, to, ch);
        }
    }
}
