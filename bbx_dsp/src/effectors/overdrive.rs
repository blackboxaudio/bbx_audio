use crate::process::Process;

pub struct OverdriveEffector;

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
