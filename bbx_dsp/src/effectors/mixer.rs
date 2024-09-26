use crate::process::Process;

pub struct MixerEffector;

impl Process for MixerEffector {
    type Sample = f32;

    fn process(&mut self, inputs: &Vec<Self::Sample>) -> Self::Sample {
        if inputs.len() > 0 {
            inputs.iter().sum::<Self::Sample>() / inputs.len() as Self::Sample
        } else {
            0.0
        }
    }
}
