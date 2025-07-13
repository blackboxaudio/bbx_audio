// SAMPLE
use std::ops::{Add, Sub, Mul, Div, AddAssign, MulAssign, DivAssign};

pub trait Sample:
    Copy + Clone + Send + Sync + 'static +
    Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> +
    AddAssign + SubAssign + MulAssign + DivAssign +
    PartialOrd + PartialEq
{
    const ZERO: Self;
    const ONE: Self;

    fn from_f64(value: f64) -> Self;
    fn to_f64(self) -> f64;
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn from_f64(value: f64) -> Self {
        value as f32
    }

    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn from_f64(value: f64) -> Self {
        value
    }

    fn to_f64(self) -> f64 {
        self
    }
}

// DSP CONTEXT
#[derive(Clone)]
pub struct DspContext {
    pub sample_rate: f64,
    pub buffer_size: usize,
    pub channels: usize,
    pub current_sample: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone)]
pub enum Parameter<S: Sample> {
    Constant(S),
    Modulated(BlockId),
}

impl<S: Sample> Parameter<S> {
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => modulation_values[block_id.0],
        }
    }
}

pub trait Block<S: Sample> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext);

    fn input_count(&self) -> usize;
    fn output_count(&self) -> usize;
    fn modulation_outputs(&self) -> &[ModulationOutput];
}

#[derive(Debug, Clone)]
pub struct ModulationOutput {
    pub name: &'static str,
    pub min_value: f64,
    pub max_value: f64,
}

// BLOCK
pub enum BlockType<S: Sample> {
    // GENERATORS
    Oscillator(OscillatorBlock<S>),

    // EFFECTORS
    Lfo(LfoBlock<S>),

    // OUTPUT
    Output(OutputBlock<S>),
}

impl<S: Sample> BlockType<S> {
    pub fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        match self {
            BlockType::Oscillator(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Lfo(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Output(block) => block.process(inputs, outputs, modulation_values, context),
        }
    }

    pub fn input_count(&self) -> usize {
        match self {
            BlockType::Oscillator(block) => block.input_count(),
            BlockType::Lfo(block) => block.input_count(),
            BlockType::Output(block) => block.input_count(),
        }
    }

    pub fn output_count(&self) -> usize {
        match self {
            BlockType::Oscillator(block) => block.output_count(),
            BlockType::Lfo(block) => block.output_count(),
            BlockType::Output(block) => block.output_count(),
        }
    }

    pub fn modulation_outputs(&self) -> &[ModulationOutput] {
        match self {
            BlockType::Oscillator(block) => block.modulation_outputs(),
            BlockType::Lfo(block) => block.modulation_outputs(),
            BlockType::Output(block) => block.modulation_outputs(),
        }
    }

    pub fn set_parameter(&mut self, parameter_name: &str, parameter: Parameter<S>) -> Result<(), String> {
        match self {
            BlockType::Oscillator(block) => {
                match parameter_name.to_lowercase().as_str() {
                    "frequency" => {
                        block.frequency = parameter;
                        Ok(())
                    },
                    _ => Err(format!("Unknown oscillator parameter: {}", parameter_name)),
                }
            },
            BlockType::Lfo(block) => {
                match parameter_name.to_lowercase().as_str() {
                    "frequency" => {
                        block.frequency = parameter;
                        Ok(())
                    },
                    "depth" => {
                        block.depth = parameter;
                        Ok(())
                    },
                    _ => Err(format!("Unknown LFO parameter: {}", parameter_name)),
                }
            },
            BlockType::Output(_) => {
                Err("Output blocks have no modulatable parameters".to_string())
            },
        }
    }
}

// GRAPH

use std::collections::HashMap;

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
        let block = BlockType::Oscillator(OscillatorBlock::new(
            S::from_f64(frequency),
            waveform,
        ));
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

        let block = BlockType::Lfo(LfoBlock {
            frequency: Parameter::Constant(S::from_f64(clamped_frequency)),
            phase: 0.0,
            waveform: Waveform::Sine,
            depth: Parameter::Constant(S::from_f64(depth)),
        });
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

// BLOCK IMPLEMENTATIONS

// OSCILLATOR
pub struct OscillatorBlock<S: Sample> {
    base_frequency: S,
    frequency: Parameter<S>,
    phase: f64,
    waveform: Waveform,
}

impl<S: Sample> OscillatorBlock<S> {
    pub fn new(frequency: S, waveform: Waveform) -> Self {
        Self {
            base_frequency: frequency,
            frequency: Parameter::Constant(frequency),
            phase: 0.0,
            waveform,
        }
    }
}

impl<S: Sample> Block<S> for OscillatorBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let freq = match &self.frequency {
            Parameter::Constant(freq) => *freq,
            Parameter::Modulated(block_id) => {
                self.base_frequency + modulation_values[block_id.0]
            }
        };

        let phase_increment = freq.to_f64() / context.sample_rate * 2.0 * std::f64::consts::PI;

        for sample_index in 0..context.buffer_size {
            let sample = match self.waveform {
                Waveform::Sine => self.phase.sin(),
                // Waveform::Square => if self.phase.sin() > 0.0 { 1.0 } else { -1.0 },
            };
            let sample_value = S::from_f64(sample);
            outputs[0][sample_index] = sample_value;
            self.phase += phase_increment;
        }

        while self.phase >= 2.0 * std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
    }

    fn input_count(&self) -> usize {
        0
    }
    fn output_count(&self) -> usize {
        1
    }
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

#[derive(Debug, Clone)]
pub enum Waveform {
    Sine,
}

// LFO
pub struct LfoBlock<S: Sample> {
    frequency: Parameter<S>,
    phase: f64,
    waveform: Waveform,
    depth: Parameter<S>,
}

impl<S: Sample> LfoBlock<S> {
    const MODULATION_OUTPUTS: &'static [ModulationOutput] = &[ModulationOutput {
        name: "LFO",
        min_value: -1.0,
        max_value: 1.0,
    }];
}

impl<S: Sample> Block<S> for LfoBlock<S> {
    fn process(&mut self, _inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext) {
        let frequency = self.frequency.get_value(modulation_values);
        let depth = self.depth.get_value(modulation_values);
        let phase_increment = frequency.to_f64() / context.sample_rate * 2.0 * std::f64::consts::PI;

        // Calculate LFO value at the start of the buffer
        let lfo_value = match self.waveform {
            Waveform::Sine => self.phase.sin(),
        } * depth.to_f64();

        let sample_value = S::from_f64(lfo_value);

        // Fill the entire buffer with this value
        // (For audio-rate modulation, you'd calculate per-sample)
        for sample_index in 0..context.buffer_size {
            outputs[0][sample_index] = sample_value;
        }

        // Advance phase by the entire buffer duration
        self.phase += phase_increment * context.buffer_size as f64;

        // Wrap phase
        while self.phase >= 2.0 * std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
    }

    fn input_count(&self) -> usize {
        0
    }
    fn output_count(&self) -> usize {
        1
    }

    fn modulation_outputs(&self) -> &[ModulationOutput] {
        Self::MODULATION_OUTPUTS
    }
}

// OUTPUT
use std::marker::PhantomData;
use std::ops::SubAssign;

pub struct OutputBlock<S: Sample> {
    channels: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sample> OutputBlock<S> {
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
            _phantom: PhantomData,
        }
    }
}

impl<S: Sample> Block<S> for OutputBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        for (input, output) in inputs.iter().zip(outputs.iter_mut()) {
            output.copy_from_slice(input);
        }
    }

    fn input_count(&self) -> usize {
        self.channels
    }
    fn output_count(&self) -> usize {
        self.channels
    }
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

// SIGNAL
use std::time::Duration;

use rodio::Source;

pub struct Signal<S: Sample> {
    graph: Graph<S>,
    output_buffers: Vec<Vec<S>>,
    channel_index: usize,
    sample_index: usize,
    channels: usize,
    buffer_size: usize,
    sample_rate: u32,
}

impl<S: Sample> Signal<S> {
    pub fn new(graph: Graph<S>) -> Self {
        let channels = graph.context.channels;
        let buffer_size = graph.context.buffer_size;
        let sample_rate = graph.context.sample_rate as u32;

        let mut output_buffers = Vec::with_capacity(channels);
        for _ in 0..channels {
            output_buffers.push(vec![S::ZERO; buffer_size]);
        }

        Self {
            graph,
            output_buffers,
            channel_index: 0,
            sample_index: 0,
            channels,
            buffer_size,
            sample_rate,
        }
    }

    fn process(&mut self) -> S {
        if self.channel_index == 0 && self.sample_index == 0 {
            let mut output_refs: Vec<&mut [S]> = self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
            self.graph.process_buffer(&mut output_refs);
        }

        let sample = self.output_buffers[self.channel_index][self.sample_index];

        // `rodio` expects interleaved samples, so we have to increment the
        // channel index every time and only increment the sample index when the
        // channel index has to be wrapped around.
        self.channel_index += 1;
        if self.channel_index >= self.channels {
            self.channel_index = 0;
            self.sample_index += 1;
            self.sample_index %= self.buffer_size;
        }

        sample
    }
}

impl<S: Sample> Iterator for Signal<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.process())
    }
}

impl Source for Signal<f32> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// PLAYER
use rodio::OutputStream;

const PLAYTIME_DURATION_SECONDS: usize = 5;

pub struct Player<S: Sample> {
    signal: Signal<S>,
}

impl<S: Sample> Player<S> {
    pub fn new(signal: Signal<S>) -> Self {
        Self { signal }
    }
}

impl Player<f32> {
    pub fn play(self, duration: Option<usize>) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let _result = stream_handle.play_raw(self.signal.convert_samples());

        std::thread::sleep(Duration::from_secs(duration.unwrap_or(PLAYTIME_DURATION_SECONDS) as u64))
    }
}

// impl Player<f64> {
//     pub fn play(self, duration: Option<usize>) {
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//
//         let f32_signal = self.signal.map(|sample| sample as f32);
//         let _result = stream_handle.play_raw(f32_signal.convert_samples());
//
//         std::thread::sleep(Duration::from_secs(
//             duration.unwrap_or(PLAYTIME_DURATION_SECONDS) as u64,
//         ))
//     }
// }
