//! DSP block system.
//!
//! This module defines the [`Block`] trait for DSP processing units and
//! [`BlockType`] for type-erased block storage in the graph.

use crate::{
    blocks::{
        effectors::{
            channel_router::ChannelRouterBlock, dc_blocker::DcBlockerBlock, gain::GainBlock,
            low_pass_filter::LowPassFilterBlock, overdrive::OverdriveBlock, panner::PannerBlock, vca::VcaBlock,
        },
        generators::oscillator::OscillatorBlock,
        io::{file_input::FileInputBlock, file_output::FileOutputBlock, output::OutputBlock},
        modulators::{envelope::EnvelopeBlock, lfo::LfoBlock},
    },
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

/// Default input count for `Effector`s.
pub(crate) const DEFAULT_EFFECTOR_INPUT_COUNT: usize = 1;
/// Default output count for `Effector`s.
pub(crate) const DEFAULT_EFFECTOR_OUTPUT_COUNT: usize = 1;

/// Default input count for `Generator`s.
pub(crate) const DEFAULT_GENERATOR_INPUT_COUNT: usize = 0;
/// Default output count for `Generator`s.
pub(crate) const DEFAULT_GENERATOR_OUTPUT_COUNT: usize = 1;

/// Default input count for `Modulator`s.
pub(crate) const DEFAULT_MODULATOR_INPUT_COUNT: usize = 0;
/// Default output count for `Modulator`s.
pub(crate) const DEFAULT_MODULATOR_OUTPUT_COUNT: usize = 1;

/// A unique identifier for a block within a DSP graph.
///
/// Used to reference blocks when creating connections or setting up modulation.
/// The inner `usize` is the block's index in the graph's block list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Category of a DSP block for visualization and organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockCategory {
    /// Audio signal generators (oscillators, noise, etc.).
    Generator,
    /// Audio signal processors (filters, effects, etc.).
    Effector,
    /// Control signal generators (LFOs, envelopes, etc.).
    Modulator,
    /// Input/output blocks (file I/O, audio output, etc.).
    IO,
}

/// The core trait for DSP processing units.
///
/// A block represents a single DSP operation (oscillator, filter, gain, etc.)
/// that processes audio buffers. Blocks are connected together in a [`Graph`](crate::graph::Graph)
/// to form a complete signal processing chain.
pub trait Block<S: Sample> {
    /// Process audio through this block.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Slice of input buffer references, one per input port
    /// * `outputs` - Slice of mutable output buffer references, one per output port
    /// * `modulation_values` - Values from connected modulator blocks, indexed by [`BlockId`]
    /// * `context` - The DSP context with sample rate and timing info
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext);

    /// Returns the number of input ports this block accepts.
    fn input_count(&self) -> usize;

    /// Returns the number of output ports this block produces.
    fn output_count(&self) -> usize;

    /// Returns the modulation outputs this block provides.
    ///
    /// Only modulator blocks (LFOs, envelopes) return non-empty slices.
    /// Generator and effector blocks return an empty slice.
    fn modulation_outputs(&self) -> &[ModulationOutput];
}

/// Type-erased container for all block implementations.
///
/// Wraps concrete block types so they can be stored uniformly in a graph.
/// Each variant corresponds to a specific DSP block type.
pub enum BlockType<S: Sample> {
    // I/O
    /// Reads audio from a file via a [`Reader`](crate::reader::Reader).
    FileInput(FileInputBlock<S>),
    /// Writes audio to a file via a [`Writer`](crate::writer::Writer).
    FileOutput(FileOutputBlock<S>),
    /// Terminal output block that collects final audio.
    Output(OutputBlock<S>),

    // GENERATORS
    /// Waveform oscillator (sine, saw, square, triangle).
    Oscillator(OscillatorBlock<S>),

    // EFFECTORS
    /// Routes channels (mono to stereo, stereo to mono, etc.).
    ChannelRouter(ChannelRouterBlock<S>),
    /// Removes DC offset from the signal.
    DcBlocker(DcBlockerBlock<S>),
    /// Adjusts signal level in decibels.
    Gain(GainBlock<S>),
    /// SVF-based low-pass filter.
    LowPassFilter(LowPassFilterBlock<S>),
    /// Asymmetric soft-clipping distortion.
    Overdrive(OverdriveBlock<S>),
    /// Stereo panning with equal-power law.
    Panner(PannerBlock<S>),
    /// Voltage controlled amplifier (multiplies audio by control signal).
    Vca(VcaBlock<S>),

    // MODULATORS
    /// ADSR envelope generator.
    Envelope(EnvelopeBlock<S>),
    /// Low-frequency oscillator for modulation.
    Lfo(LfoBlock<S>),
}

impl<S: Sample> BlockType<S> {
    /// Perform the calculation of the underlying `Block`.
    #[inline]
    pub fn process(
        &mut self,
        inputs: &[&[S]],
        outputs: &mut [&mut [S]],
        modulation_values: &[S],
        context: &DspContext,
    ) {
        match self {
            // I/O
            BlockType::FileInput(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::FileOutput(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Output(block) => block.process(inputs, outputs, modulation_values, context),

            // GENERATORS
            BlockType::Oscillator(block) => block.process(inputs, outputs, modulation_values, context),

            // EFFECTORS
            BlockType::ChannelRouter(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::DcBlocker(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Gain(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::LowPassFilter(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Overdrive(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Panner(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Vca(block) => block.process(inputs, outputs, modulation_values, context),

            // MODULATORS
            BlockType::Envelope(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Lfo(block) => block.process(inputs, outputs, modulation_values, context),
        }
    }

    /// Get the input count of the underlying `Block`.
    #[inline]
    pub fn input_count(&self) -> usize {
        match self {
            // I/O
            BlockType::FileInput(block) => block.input_count(),
            BlockType::FileOutput(block) => block.input_count(),
            BlockType::Output(block) => block.input_count(),

            // GENERATORS
            BlockType::Oscillator(block) => block.input_count(),

            // EFFECTORS
            BlockType::ChannelRouter(block) => block.input_count(),
            BlockType::DcBlocker(block) => block.input_count(),
            BlockType::Gain(block) => block.input_count(),
            BlockType::LowPassFilter(block) => block.input_count(),
            BlockType::Overdrive(block) => block.input_count(),
            BlockType::Panner(block) => block.input_count(),
            BlockType::Vca(block) => block.input_count(),

            // MODULATORS
            BlockType::Envelope(block) => block.input_count(),
            BlockType::Lfo(block) => block.input_count(),
        }
    }

    /// Get the output count of the underlying `Block`.
    #[inline]
    pub fn output_count(&self) -> usize {
        match self {
            // I/O
            BlockType::FileInput(block) => block.output_count(),
            BlockType::FileOutput(block) => block.output_count(),
            BlockType::Output(block) => block.output_count(),

            // GENERATORS
            BlockType::Oscillator(block) => block.output_count(),

            // EFFECTORS
            BlockType::ChannelRouter(block) => block.output_count(),
            BlockType::DcBlocker(block) => block.output_count(),
            BlockType::Gain(block) => block.output_count(),
            BlockType::LowPassFilter(block) => block.output_count(),
            BlockType::Overdrive(block) => block.output_count(),
            BlockType::Panner(block) => block.output_count(),
            BlockType::Vca(block) => block.output_count(),

            // MODULATORS
            BlockType::Envelope(block) => block.output_count(),
            BlockType::Lfo(block) => block.output_count(),
        }
    }

    /// Get the modulation outputs (if any) of the underlying `Block`.
    #[inline]
    pub fn modulation_outputs(&self) -> &[ModulationOutput] {
        match self {
            // I/O
            BlockType::FileInput(block) => block.modulation_outputs(),
            BlockType::FileOutput(block) => block.modulation_outputs(),
            BlockType::Output(block) => block.modulation_outputs(),

            // GENERATORS
            BlockType::Oscillator(block) => block.modulation_outputs(),

            // EFFECTORS
            BlockType::ChannelRouter(block) => block.modulation_outputs(),
            BlockType::DcBlocker(block) => block.modulation_outputs(),
            BlockType::Gain(block) => block.modulation_outputs(),
            BlockType::LowPassFilter(block) => block.modulation_outputs(),
            BlockType::Overdrive(block) => block.modulation_outputs(),
            BlockType::Panner(block) => block.modulation_outputs(),
            BlockType::Vca(block) => block.modulation_outputs(),

            // MODULATORS
            BlockType::Envelope(block) => block.modulation_outputs(),
            BlockType::Lfo(block) => block.modulation_outputs(),
        }
    }

    /// Set a given `Parameter` of the underlying `Block`.
    pub fn set_parameter(&mut self, parameter_name: &str, parameter: Parameter<S>) -> Result<(), String> {
        match self {
            // I/O
            BlockType::FileInput(_) => Err("File input blocks have no modulated parameters".to_string()),
            BlockType::FileOutput(_) => Err("File output blocks have no modulated parameters".to_string()),
            BlockType::Output(_) => Err("Output blocks have no modulated parameters".to_string()),

            // GENERATORS
            BlockType::Oscillator(block) => match parameter_name.to_lowercase().as_str() {
                "frequency" => {
                    block.frequency = parameter;
                    Ok(())
                }
                "pitch_offset" => {
                    block.pitch_offset = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown oscillator parameter: {parameter_name}")),
            },

            // EFFECTORS
            BlockType::ChannelRouter(_) => Err("Channel router uses direct field access, not Parameter<S>".to_string()),
            BlockType::DcBlocker(_) => Err("DC blocker uses direct field access, not Parameter<S>".to_string()),
            BlockType::Gain(block) => match parameter_name.to_lowercase().as_str() {
                "level" | "level_db" => {
                    block.level_db = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown gain parameter: {parameter_name}")),
            },
            BlockType::LowPassFilter(block) => match parameter_name.to_lowercase().as_str() {
                "cutoff" | "frequency" => {
                    block.cutoff = parameter;
                    Ok(())
                }
                "resonance" | "q" => {
                    block.resonance = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown low-pass filter parameter: {parameter_name}")),
            },
            BlockType::Overdrive(block) => match parameter_name.to_lowercase().as_str() {
                "drive" => {
                    block.drive = parameter;
                    Ok(())
                }
                "level" => {
                    block.level = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown overdrive parameter: {parameter_name}")),
            },
            BlockType::Panner(block) => match parameter_name.to_lowercase().as_str() {
                "position" | "pan" => {
                    block.position = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown panner parameter: {parameter_name}")),
            },
            BlockType::Vca(_) => Err("VCA has no modulated parameters".to_string()),

            // MODULATORS
            BlockType::Envelope(block) => match parameter_name.to_lowercase().as_str() {
                "attack" => {
                    block.attack = parameter;
                    Ok(())
                }
                "decay" => {
                    block.decay = parameter;
                    Ok(())
                }
                "sustain" => {
                    block.sustain = parameter;
                    Ok(())
                }
                "release" => {
                    block.release = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown envelope parameter: {parameter_name}")),
            },
            BlockType::Lfo(block) => match parameter_name.to_lowercase().as_str() {
                "frequency" => {
                    block.frequency = parameter;
                    Ok(())
                }
                "depth" => {
                    block.depth = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown LFO parameter: {parameter_name}")),
            },
        }
    }

    /// Returns `true` if this block is a modulator (LFO or Envelope).
    #[inline]
    pub fn is_modulator(&self) -> bool {
        matches!(self, BlockType::Envelope(_) | BlockType::Lfo(_))
    }

    /// Returns `true` if this block is an output-type block (Output or FileOutput).
    #[inline]
    pub fn is_output(&self) -> bool {
        matches!(self, BlockType::Output(_) | BlockType::FileOutput(_))
    }

    /// Returns the category of this block.
    #[inline]
    pub fn category(&self) -> BlockCategory {
        match self {
            BlockType::FileInput(_) | BlockType::FileOutput(_) | BlockType::Output(_) => BlockCategory::IO,
            BlockType::Oscillator(_) => BlockCategory::Generator,
            BlockType::ChannelRouter(_)
            | BlockType::DcBlocker(_)
            | BlockType::Gain(_)
            | BlockType::LowPassFilter(_)
            | BlockType::Overdrive(_)
            | BlockType::Panner(_)
            | BlockType::Vca(_) => BlockCategory::Effector,
            BlockType::Envelope(_) | BlockType::Lfo(_) => BlockCategory::Modulator,
        }
    }

    /// Returns the display name of this block type.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            BlockType::FileInput(_) => "File Input",
            BlockType::FileOutput(_) => "File Output",
            BlockType::Output(_) => "Output",
            BlockType::Oscillator(_) => "Oscillator",
            BlockType::ChannelRouter(_) => "Channel Router",
            BlockType::DcBlocker(_) => "DC Blocker",
            BlockType::Gain(_) => "Gain",
            BlockType::LowPassFilter(_) => "Low Pass Filter",
            BlockType::Overdrive(_) => "Overdrive",
            BlockType::Panner(_) => "Panner",
            BlockType::Vca(_) => "VCA",
            BlockType::Envelope(_) => "Envelope",
            BlockType::Lfo(_) => "LFO",
        }
    }

    /// Returns all modulated parameters and their source block IDs.
    ///
    /// Returns a list of (parameter_name, source_block_id) for each parameter
    /// that is modulated by another block.
    pub fn get_modulated_parameters(&self) -> Vec<(&'static str, BlockId)> {
        let mut result = Vec::new();

        match self {
            BlockType::FileInput(_) | BlockType::FileOutput(_) | BlockType::Output(_) => {}

            BlockType::Oscillator(block) => {
                if let Parameter::Modulated(id) = &block.frequency {
                    result.push(("frequency", *id));
                }
                if let Parameter::Modulated(id) = &block.pitch_offset {
                    result.push(("pitch_offset", *id));
                }
            }

            BlockType::ChannelRouter(_) | BlockType::DcBlocker(_) | BlockType::Vca(_) => {}

            BlockType::Gain(block) => {
                if let Parameter::Modulated(id) = &block.level_db {
                    result.push(("level", *id));
                }
            }

            BlockType::LowPassFilter(block) => {
                if let Parameter::Modulated(id) = &block.cutoff {
                    result.push(("cutoff", *id));
                }
                if let Parameter::Modulated(id) = &block.resonance {
                    result.push(("resonance", *id));
                }
            }

            BlockType::Overdrive(block) => {
                if let Parameter::Modulated(id) = &block.drive {
                    result.push(("drive", *id));
                }
                if let Parameter::Modulated(id) = &block.level {
                    result.push(("level", *id));
                }
            }

            BlockType::Panner(block) => {
                if let Parameter::Modulated(id) = &block.position {
                    result.push(("position", *id));
                }
            }

            BlockType::Envelope(block) => {
                if let Parameter::Modulated(id) = &block.attack {
                    result.push(("attack", *id));
                }
                if let Parameter::Modulated(id) = &block.decay {
                    result.push(("decay", *id));
                }
                if let Parameter::Modulated(id) = &block.sustain {
                    result.push(("sustain", *id));
                }
                if let Parameter::Modulated(id) = &block.release {
                    result.push(("release", *id));
                }
            }

            BlockType::Lfo(block) => {
                if let Parameter::Modulated(id) = &block.frequency {
                    result.push(("frequency", *id));
                }
                if let Parameter::Modulated(id) = &block.depth {
                    result.push(("depth", *id));
                }
            }
        }

        result
    }
}
