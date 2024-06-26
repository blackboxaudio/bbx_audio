use rand::Rng;

use crate::sample::Sample;

pub struct Graph {
    sample_rate: usize,
}

impl Graph {
    pub fn new(sample_rate: usize) -> Graph {
        return Graph {
            sample_rate,
        }
    }
}

impl Graph {
    pub fn evaluate(&self) -> Sample<f32> {
        let mut rng = rand::thread_rng();
        return rng.gen::<f32>();
    }
}