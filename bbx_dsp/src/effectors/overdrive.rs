use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

pub struct OverdriveEffector;

impl Display for OverdriveEffector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_Overdrive")
    }
}

impl Process for OverdriveEffector {
    #[inline]
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
