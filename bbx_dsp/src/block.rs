use crate::{
    blocks::{
        effectors::{
            dc_blocker::DcBlockerBlock,
            filter::FilterBlock,
            gain::GainBlock,
            overdrive::OverdriveBlock,
            panner::PannerBlock,
        },
        generators::oscillator::OscillatorBlock,
        io::{file_input::FileInputBlock, file_output::FileOutputBlock, output::OutputBlock},
        modulators::lfo::LfoBlock,
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

/// Used to identify and find blocks within a DSP `Graph`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Describes a structure for a particular DSP operation.
pub trait Block<S: Sample> {
    /// Perform the calculation of a particular `Block`.
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext);

    /// Get the input count of a `Block`.
    fn input_count(&self) -> usize;

    /// Get the output count of a `Block`.
    fn output_count(&self) -> usize;

    /// Get the modulation outputs (if any) of a `Block`.
    fn modulation_outputs(&self) -> &[ModulationOutput];
}

/// Supported types of blocks i.e. DSP operations
/// that can be used within a `Graph`.
pub enum BlockType<S: Sample> {
    // I/O
    FileInput(FileInputBlock<S>),
    FileOutput(FileOutputBlock<S>),
    Output(OutputBlock<S>),

    // GENERATORS
    Oscillator(OscillatorBlock<S>),

    // EFFECTORS
    DcBlocker(DcBlockerBlock<S>),
    Filter(FilterBlock<S>),
    Gain(GainBlock<S>),
    Overdrive(OverdriveBlock<S>),
    Panner(PannerBlock<S>),

    // MODULATORS
    Lfo(LfoBlock<S>),
}

impl<S: Sample> BlockType<S> {
    /// Perform the calculation of the underlying `Block`.
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
            BlockType::DcBlocker(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Filter(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Gain(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Overdrive(block) => block.process(inputs, outputs, modulation_values, context),
            BlockType::Panner(block) => block.process(inputs, outputs, modulation_values, context),

            // MODULATORS
            BlockType::Lfo(block) => block.process(inputs, outputs, modulation_values, context),
        }
    }

    /// Get the input count of the underlying `Block`.
    pub fn input_count(&self) -> usize {
        match self {
            // I/O
            BlockType::FileInput(block) => block.input_count(),
            BlockType::FileOutput(block) => block.input_count(),
            BlockType::Output(block) => block.input_count(),

            // GENERATORS
            BlockType::Oscillator(block) => block.input_count(),

            // EFFECTORS
            BlockType::DcBlocker(block) => block.input_count(),
            BlockType::Filter(block) => block.input_count(),
            BlockType::Gain(block) => block.input_count(),
            BlockType::Overdrive(block) => block.input_count(),
            BlockType::Panner(block) => block.input_count(),

            // MODULATORS
            BlockType::Lfo(block) => block.input_count(),
        }
    }

    /// Get the output count of the underlying `Block`.
    pub fn output_count(&self) -> usize {
        match self {
            // I/O
            BlockType::FileInput(block) => block.output_count(),
            BlockType::FileOutput(block) => block.output_count(),
            BlockType::Output(block) => block.output_count(),

            // GENERATORS
            BlockType::Oscillator(block) => block.output_count(),

            // EFFECTORS
            BlockType::DcBlocker(block) => block.output_count(),
            BlockType::Filter(block) => block.output_count(),
            BlockType::Gain(block) => block.output_count(),
            BlockType::Overdrive(block) => block.output_count(),
            BlockType::Panner(block) => block.output_count(),

            // MODULATORS
            BlockType::Lfo(block) => block.output_count(),
        }
    }

    /// Get the modulation outputs (if any) of the underlying `Block`.
    pub fn modulation_outputs(&self) -> &[ModulationOutput] {
        match self {
            // I/O
            BlockType::FileInput(block) => block.modulation_outputs(),
            BlockType::FileOutput(block) => block.modulation_outputs(),
            BlockType::Output(block) => block.modulation_outputs(),

            // GENERATORS
            BlockType::Oscillator(block) => block.modulation_outputs(),

            // EFFECTORS
            BlockType::DcBlocker(block) => block.modulation_outputs(),
            BlockType::Filter(block) => block.modulation_outputs(),
            BlockType::Gain(block) => block.modulation_outputs(),
            BlockType::Overdrive(block) => block.modulation_outputs(),
            BlockType::Panner(block) => block.modulation_outputs(),

            // MODULATORS
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
                _ => Err(format!("Unknown oscillator parameter: {parameter_name}")),
            },

            // EFFECTORS
            BlockType::DcBlocker(_) => Err("DC blocker has no modulatable parameters".to_string()),

            BlockType::Filter(block) => match parameter_name.to_lowercase().as_str() {
                "cutoff" | "frequency" => {
                    Self::apply_to_modulatable(&mut block.cutoff, parameter);
                    Ok(())
                }
                "resonance" | "q" => {
                    Self::apply_to_modulatable(&mut block.resonance, parameter);
                    Ok(())
                }
                _ => Err(format!("Unknown filter parameter: {parameter_name}")),
            },

            BlockType::Gain(block) => match parameter_name.to_lowercase().as_str() {
                "level" | "gain" => {
                    Self::apply_to_modulatable(&mut block.level, parameter);
                    Ok(())
                }
                _ => Err(format!("Unknown gain parameter: {parameter_name}")),
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
                    Self::apply_to_modulatable(&mut block.position, parameter);
                    Ok(())
                }
                _ => Err(format!("Unknown panner parameter: {parameter_name}")),
            },

            // MODULATORS
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

    /// Apply a Parameter to a ModulatableParam.
    fn apply_to_modulatable<const N: usize>(
        target: &mut crate::parameter::ModulatableParam<S, N>,
        parameter: Parameter<S>,
    ) {
        match parameter {
            Parameter::Constant(value) => target.set(value),
            Parameter::Modulated(source_id) => {
                target.add_modulation(source_id, S::ONE);
            }
            Parameter::External(ptr) => target.bind_external(ptr),
        }
    }
}
