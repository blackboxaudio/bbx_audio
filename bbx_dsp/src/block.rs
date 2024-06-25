use rand::Rng;
use crate::sample::Sample;

pub trait Process<T> {
    fn process(&self, sample: Option<T>) -> T;
}

pub struct Block {
    pub next_block: Option<Box<Block>>,
}

impl Process<Sample<f32>> for Block {
    fn process(&self, _sample: Option<Sample<f32>>) -> Sample<f32> {
        let mut rng = rand::thread_rng();
        return rng.gen::<f32>();
    }
}



