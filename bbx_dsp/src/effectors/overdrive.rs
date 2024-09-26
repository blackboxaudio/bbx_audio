use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

pub struct OverdriveEffector;

impl Display for OverdriveEffector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_Overdrive")
    }
}

impl Process for OverdriveEffector {
    type Sample = f32;

    fn process(&mut self, inputs: &Vec<Self::Sample>) -> Self::Sample {
        let input_sum = inputs.iter().sum::<Self::Sample>() / inputs.len() as Self::Sample;
        if input_sum > 1.0 {
            1.0
        } else if input_sum < -1.0 {
            -1.0
        } else {
            input_sum - (input_sum.powi(3) / 3.0)
        }
    }
}
