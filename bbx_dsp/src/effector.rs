use crate::{process::Process, sample::Sample};

pub struct Effector;

impl Effector {
    pub fn new() -> Effector {
        return Effector {};
    }
}

impl Process<Sample<f32>> for Effector {
    fn process(&mut self, sample: Option<Sample<f32>>) -> Sample<f32> {
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
