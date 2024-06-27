use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

pub struct MixerEffector;

impl Display for MixerEffector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_Mixer")
    }
}

impl Process for MixerEffector {
    #[inline]
    fn process(&mut self, sample: Option<Sample>) -> Sample {
        return if let Some(sample_value) = sample {
            sample_value
        } else {
            0.0
        };
    }
}
