use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

pub struct MixerEffector;

impl Display for MixerEffector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_Mixer")
    }
}

impl Process for MixerEffector {
    fn process(&mut self, inputs: &Vec<Sample>) -> Sample {
        if inputs.len() > 0 {
            inputs.iter().sum::<Sample>() / inputs.len() as Sample
        } else {
            0.0
        }
    }
}
