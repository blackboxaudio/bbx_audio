//! Voltage Controlled Amplifier (VCA) block.
//!
//! Multiplies an audio signal by a control signal, typically from an envelope.

use crate::{block::Block, context::DspContext, parameter::ModulationOutput, sample::Sample};

/// A voltage controlled amplifier that multiplies audio by a control signal.
///
/// The VCA takes two inputs:
/// - Input 0: Audio signal
/// - Input 1: Control signal (typically 0.0 to 1.0 from an envelope)
///
/// The output is the sample-by-sample product of both inputs.
pub struct VcaBlock<S: Sample> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: Sample> VcaBlock<S> {
    /// Create a new VCA block.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: Sample> Default for VcaBlock<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Sample> Block<S> for VcaBlock<S> {
    fn process(&mut self, inputs: &[&[S]], outputs: &mut [&mut [S]], _modulation_values: &[S], _context: &DspContext) {
        let output = match outputs.first_mut() {
            Some(out) => out,
            None => return,
        };

        let audio_input = inputs.first().copied().unwrap_or(&[]);
        let control_input = inputs.get(1).copied().unwrap_or(&[]);

        for (i, out_sample) in output.iter_mut().enumerate() {
            let audio = audio_input.get(i).copied().unwrap_or(S::ZERO);
            let control = control_input.get(i).copied().unwrap_or(S::ONE);
            *out_sample = audio * control;
        }
    }

    #[inline]
    fn input_count(&self) -> usize {
        2
    }

    #[inline]
    fn output_count(&self) -> usize {
        1
    }

    #[inline]
    fn modulation_outputs(&self) -> &[ModulationOutput] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DspContext;

    fn test_context(buffer_size: usize) -> DspContext {
        DspContext {
            sample_rate: 44100.0,
            num_channels: 1,
            buffer_size,
            current_sample: 0,
        }
    }

    #[test]
    fn test_vca_multiplication() {
        let mut vca = VcaBlock::<f32>::new();
        let context = test_context(4);

        let audio = [1.0f32, 0.5, -0.5, -1.0];
        let control = [1.0f32, 0.5, 0.5, 0.0];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 2] = [&audio, &control];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        vca.process(&inputs, &mut outputs, &[], &context);

        assert!((output[0] - 1.0).abs() < 1e-6);
        assert!((output[1] - 0.25).abs() < 1e-6);
        assert!((output[2] - -0.25).abs() < 1e-6);
        assert!((output[3] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vca_missing_control_defaults_to_unity() {
        let mut vca = VcaBlock::<f32>::new();
        let context = test_context(4);

        let audio = [0.5f32, 0.5, 0.5, 0.5];
        let mut output = [0.0f32; 4];

        let inputs: [&[f32]; 1] = [&audio];
        let mut outputs: [&mut [f32]; 1] = [&mut output];

        vca.process(&inputs, &mut outputs, &[], &context);

        for sample in output.iter() {
            assert!((sample - 0.5).abs() < 1e-6);
        }
    }
}
