use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

/// A type of DSP `Block` that produces an output signal by modifying an input signal.
pub struct Effector;

impl Effector {
    pub fn new() -> Effector {
        return Effector {};
    }
}

impl Display for Effector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Effector")
    }
}

impl Process for Effector {
    fn process(&mut self, sample: Option<Sample>) -> Sample {
        return if let Some(sample_result) = sample {
            if sample_result > 1.0 {
                1.0
            } else if sample_result < -1.0 {
                -1.0
            } else {
                sample_result - (sample_result.powi(3) / 3.0)
            }
        } else {
            0.0
        };
    }
}
