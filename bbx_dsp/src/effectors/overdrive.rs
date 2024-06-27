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
    fn process(&mut self, inputs: &Vec<Sample>) -> Sample {
        let input_sum: Sample = inputs.iter().sum::<Sample>() / inputs.len() as Sample;
        return if input_sum > 1.0 {
            1.0
        } else if input_sum < -1.0 {
            -1.0
        } else {
            input_sum - (input_sum.powi(3) / 3.0)
        };
    }
}
