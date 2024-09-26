use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

pub struct MixerEffector;

impl Display for MixerEffector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_Mixer")
    }
}

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
