//! DSP graph system.
//!
//! This module provides [`Graph`] for managing connected DSP blocks and
//! [`GraphBuilder`] for fluent graph construction.
//!
//! Blocks are connected to form a signal processing chain. The graph handles
//! buffer allocation, execution ordering via topological sort, and modulation
//! value collection.

use std::collections::HashMap;

use bbx_core::StackVec;

use crate::{
    block::{BlockCategory, BlockId, BlockType},
    blocks::{
        effectors::{
            ambisonic_decoder::AmbisonicDecoderBlock,
            binaural_decoder::BinauralDecoderBlock,
            channel_merger::ChannelMergerBlock,
            channel_router::{ChannelMode, ChannelRouterBlock},
            channel_splitter::ChannelSplitterBlock,
            dc_blocker::DcBlockerBlock,
            gain::GainBlock,
            low_pass_filter::LowPassFilterBlock,
            matrix_mixer::MatrixMixerBlock,
            overdrive::OverdriveBlock,
            panner::PannerBlock,
            vca::VcaBlock,
        },
        generators::oscillator::OscillatorBlock,
        io::{file_input::FileInputBlock, file_output::FileOutputBlock, output::OutputBlock},
        modulators::{envelope::EnvelopeBlock, lfo::LfoBlock},
    },
    buffer::{AudioBuffer, Buffer},
    channel::ChannelLayout,
    context::DspContext,
    parameter::Parameter,
    reader::Reader,
    sample::Sample,
    waveform::Waveform,
    writer::Writer,
};

/// Maximum number of inputs a block can have (realtime-safe stack allocation).
/// Set to 16 to support third-order ambisonics (16 channels).
pub const MAX_BLOCK_INPUTS: usize = 16;
/// Maximum number of outputs a block can have (realtime-safe stack allocation).
/// Set to 16 to support third-order ambisonics (16 channels).
pub const MAX_BLOCK_OUTPUTS: usize = 16;

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
    /// Target block ID.
    pub to_block: usize,
    /// Name of the modulated parameter on the target block.
    pub parameter_name: String,
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
    audio_buffers: Vec<AudioBuffer<S>>,
    modulation_values: Vec<S>,

    // Buffer management
    block_buffer_start: Vec<usize>,
    buffer_size: usize,
    context: DspContext,

    // Pre-computed connection lookups: block_id -> [input buffer indices]
    // Computed once in prepare_for_playback() for O(1) lookup during processing
    block_input_buffers: Vec<Vec<usize>>,
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

    /// Prepares the graph for audio processing.
    ///
    /// Must be called after all blocks are added and connected, but before
    /// [`process_buffers`](Self::process_buffers). Computes execution order
    /// and pre-allocates buffers.
    pub fn prepare_for_playback(&mut self) {
        self.execution_order = self.topological_sort();
        self.modulation_values.resize(self.blocks.len(), S::ZERO);

        // Pre-compute input buffer indices for each block (O(1) lookup during processing)
        self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
        for conn in &self.connections {
            let buffer_idx = self.get_buffer_index(conn.from, conn.from_output);
            self.block_input_buffers[conn.to.0].push(buffer_idx);
        }

        #[cfg(debug_assertions)]
        self.validate_buffer_indices();
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
                    !input_indices.contains(&buffer_idx),
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
            for &index in input_indices {
                let buffer_ptr = buffers_ptr.add(index);
                let slice = std::slice::from_raw_parts((*buffer_ptr).as_ptr(), (*buffer_ptr).len());
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
                &self.modulation_values,
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
    fn collect_modulation_values(&mut self, block_id: BlockId) {
        // Bounds check to prevent panic in audio thread
        if block_id.0 >= self.blocks.len() {
            return;
        }

        let has_modulation = !self.blocks[block_id.0].modulation_outputs().is_empty();
        if has_modulation {
            let buffer_index = self.get_buffer_index(block_id, 0);
            // Take only the first sample (control rate, not audio rate)
            if let (Some(&first_sample), Some(mod_val)) = (
                self.audio_buffers.get(buffer_index).and_then(|b| b.as_slice().first()),
                self.modulation_values.get_mut(block_id.0),
            ) {
                *mod_val = first_sample;
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

    /// Add a pre-configured block to the graph.
    ///
    /// Use this when you need to configure a block before adding it,
    /// such as setting matrix mixer gains or other complex initialization.
    pub fn add_block(&mut self, block: BlockType<S>) -> BlockId {
        self.graph.add_block(block)
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

    /// Add a `VcaBlock` to the `Graph`.
    ///
    /// The VCA multiplies audio (input 0) by a control signal (input 1).
    /// Typically used with an envelope for amplitude modulation.
    pub fn add_vca(&mut self) -> BlockId {
        let block = BlockType::Vca(VcaBlock::new());
        self.graph.add_block(block)
    }

    /// Add a `GainBlock` to the `Graph`.
    ///
    /// Level is specified in decibels (dB), clamped to -80 to +30 dB.
    /// An optional base gain multiplier (linear) can be applied in addition to the dB level.
    pub fn add_gain(&mut self, level_db: f64, base_gain: Option<f64>) -> BlockId {
        let block = BlockType::Gain(GainBlock::new(
            S::from_f64(level_db),
            Some(S::from_f64(base_gain.unwrap_or(<f64 as Sample>::ONE))),
        ));
        self.graph.add_block(block)
    }

    /// Add a `LowPassFilterBlock` to the `Graph`.
    ///
    /// Uses SVF (State Variable Filter) topology for stable filtering.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - Cutoff frequency in Hz (clamped to 20-20000 Hz)
    /// * `resonance` - Q factor (clamped to 0.5-10.0, default 0.707 is Butterworth)
    pub fn add_low_pass_filter(&mut self, cutoff: f64, resonance: f64) -> BlockId {
        let block = BlockType::LowPassFilter(LowPassFilterBlock::new(S::from_f64(cutoff), S::from_f64(resonance)));
        self.graph.add_block(block)
    }

    /// Add a `MatrixMixerBlock` to the `Graph`.
    ///
    /// Creates an NxM mixing matrix for flexible channel routing.
    /// Use `set_gain()` on the block to configure routing weights.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Number of input channels (1-16)
    /// * `outputs` - Number of output channels (1-16)
    pub fn add_matrix_mixer(&mut self, inputs: usize, outputs: usize) -> BlockId {
        let block = BlockType::MatrixMixer(MatrixMixerBlock::new(inputs, outputs));
        self.graph.add_block(block)
    }

    /// Add a `ChannelSplitterBlock` to the `Graph`.
    ///
    /// Splits multi-channel input into individual mono outputs.
    ///
    /// # Arguments
    ///
    /// * `channels` - Number of channels to split (1-16)
    pub fn add_channel_splitter(&mut self, channels: usize) -> BlockId {
        let block = BlockType::ChannelSplitter(ChannelSplitterBlock::new(channels));
        self.graph.add_block(block)
    }

    /// Add a `ChannelMergerBlock` to the `Graph`.
    ///
    /// Merges individual mono inputs into a multi-channel output.
    ///
    /// # Arguments
    ///
    /// * `channels` - Number of channels to merge (1-16)
    pub fn add_channel_merger(&mut self, channels: usize) -> BlockId {
        let block = BlockType::ChannelMerger(ChannelMergerBlock::new(channels));
        self.graph.add_block(block)
    }

    /// Add a stereo `PannerBlock` to the `Graph`.
    ///
    /// Uses constant-power pan law for smooth stereo positioning.
    ///
    /// # Arguments
    ///
    /// * `position` - Pan position from -100 (left) to +100 (right), 0 = center
    pub fn add_panner_stereo(&mut self, position: f64) -> BlockId {
        let block = BlockType::Panner(PannerBlock::new(S::from_f64(position)));
        self.graph.add_block(block)
    }

    /// Add a surround `PannerBlock` to the `Graph`.
    ///
    /// Uses VBAP (Vector Base Amplitude Panning) for surround layouts.
    /// Control azimuth and elevation parameters for positioning.
    ///
    /// # Arguments
    ///
    /// * `layout` - Target speaker layout (Surround51 or Surround71)
    pub fn add_panner_surround(&mut self, layout: ChannelLayout) -> BlockId {
        let block = BlockType::Panner(PannerBlock::new_surround(layout));
        self.graph.add_block(block)
    }

    /// Add an ambisonic encoder `PannerBlock` to the `Graph`.
    ///
    /// Encodes mono input to SN3D normalized, ACN ordered B-format.
    /// Control azimuth and elevation parameters for source positioning.
    ///
    /// # Arguments
    ///
    /// * `order` - Ambisonic order (1 = FOA/4ch, 2 = SOA/9ch, 3 = TOA/16ch)
    pub fn add_panner_ambisonic(&mut self, order: usize) -> BlockId {
        let block = BlockType::Panner(PannerBlock::new_ambisonic(order));
        self.graph.add_block(block)
    }

    /// Add an `AmbisonicDecoderBlock` to the `Graph`.
    ///
    /// Decodes ambisonics B-format to a speaker layout using mode-matching.
    ///
    /// # Arguments
    ///
    /// * `order` - Ambisonic order (1, 2, or 3)
    /// * `output_layout` - Target speaker layout for decoding
    pub fn add_ambisonic_decoder(&mut self, order: usize, output_layout: ChannelLayout) -> BlockId {
        let block = BlockType::AmbisonicDecoder(AmbisonicDecoderBlock::new(order, output_layout));
        self.graph.add_block(block)
    }

    /// Add a `BinauralDecoderBlock` to the `Graph`.
    ///
    /// Decodes ambisonics B-format to stereo for headphone listening using
    /// psychoacoustically-informed matrix coefficients.
    ///
    /// # Arguments
    ///
    /// * `order` - Ambisonic order (1, 2, or 3)
    pub fn add_binaural_decoder(&mut self, order: usize) -> BlockId {
        let block = BlockType::BinauralDecoder(BinauralDecoderBlock::new(order));
        self.graph.add_block(block)
    }

    /// Add a `DcBlockerBlock` to the `Graph`.
    ///
    /// Removes DC offset from audio signals using a first-order high-pass filter
    /// with approximately 5Hz cutoff. Useful after distortion effects.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the DC blocker is active
    pub fn add_dc_blocker(&mut self, enabled: bool) -> BlockId {
        let block = BlockType::DcBlocker(DcBlockerBlock::new(enabled));
        self.graph.add_block(block)
    }

    /// Add a `ChannelRouterBlock` to the `Graph`.
    ///
    /// Routes and manipulates stereo signals with channel selection, mono summing,
    /// and phase inversion.
    ///
    /// # Arguments
    ///
    /// * `mode` - Channel routing mode (Stereo, Left, Right, Swap)
    /// * `mono` - Sum to mono (L+R)/2 on both channels
    /// * `invert_left` - Invert left channel phase
    /// * `invert_right` - Invert right channel phase
    pub fn add_channel_router(
        &mut self,
        mode: ChannelMode,
        mono: bool,
        invert_left: bool,
        invert_right: bool,
    ) -> BlockId {
        let block = BlockType::ChannelRouter(ChannelRouterBlock::new(mode, mono, invert_left, invert_right));
        self.graph.add_block(block)
    }

    // MODULATORS

    /// Add an `EnvelopeBlock` to the `Graph`, which is useful for ADSR-style
    /// amplitude or parameter modulation.
    pub fn add_envelope(&mut self, attack: f64, decay: f64, sustain: f64, release: f64) -> BlockId {
        let block = BlockType::Envelope(EnvelopeBlock::new(
            S::from_f64(attack),
            S::from_f64(decay),
            S::from_f64(sustain.clamp(0.0, 1.0)),
            S::from_f64(release),
        ));
        self.graph.add_block(block)
    }

    /// Add an `LfoBlock` to the `Graph`, which is useful when wanting to
    /// modulate one or more `Parameter`s of one or more blocks.
    ///
    /// # Control-Rate Limitation
    ///
    /// LFO frequency is clamped to `sample_rate / (2 * buffer_size)` because modulation
    /// operates at control rate (per-buffer), not audio rate. At 44.1kHz with 512 samples,
    /// max frequency is ~43Hz. See [`Graph::collect_modulation_values`] for details.
    pub fn add_lfo(&mut self, frequency: f64, depth: f64, seed: Option<u64>) -> BlockId {
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
                    .get_modulated_parameters()
                    .into_iter()
                    .map(move |(param_name, source_id)| ModulationConnectionSnapshot {
                        from_block: source_id.0,
                        to_block: target_id,
                        parameter_name: param_name.to_string(),
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
    /// # Panics
    ///
    /// Panics if any block has more inputs or outputs than the realtime-safe
    /// limits (`MAX_BLOCK_INPUTS` or `MAX_BLOCK_OUTPUTS`).
    pub fn build(mut self) -> Graph<S> {
        let output = self.graph.add_output_block();

        // Find all terminal blocks: blocks with no outgoing connections,
        // excluding modulators (LFO, Envelope) and output-type blocks.
        // These terminal blocks will all be mixed to the output.
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

        // Connect all terminal blocks to output (signals will be summed)
        for block_id in terminal_blocks {
            for channel_index in 0..self.graph.context.num_channels {
                self.connect(block_id, 0, output, channel_index);
            }
        }

        self.graph.prepare_for_playback();

        // Validate that all blocks are within realtime-safe I/O limits
        // This must run after prepare_for_playback() to check actual connection counts
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
}
