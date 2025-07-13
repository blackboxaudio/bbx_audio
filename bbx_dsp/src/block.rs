use crate::{
    blocks::{generators::oscillator::OscillatorBlock, modulators::lfo::LfoBlock, output::OutputBlock},
    context::DspContext,
    parameter::{ModulationOutput, Parameter},
    sample::Sample,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

pub trait Block<S: Sample> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], modulation_values: &[S], context: &DspContext);

    fn input_count(&self) -> usize;
    fn output_count(&self) -> usize;
    fn modulation_outputs(&self) -> &[ModulationOutput];
}

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
            BlockType::Oscillator(block) => match parameter_name.to_lowercase().as_str() {
                "frequency" => {
                    block.frequency = parameter;
                    Ok(())
                }
                _ => Err(format!("Unknown oscillator parameter: {parameter_name}")),
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
            BlockType::Output(_) => Err("Output blocks have no modulatable parameters".to_string()),
        }
    }
}
