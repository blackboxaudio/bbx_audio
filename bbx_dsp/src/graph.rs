use std::time::Duration;
use rand::Rng;
use rodio::Source;
use crate::sample::Sample;

pub trait Process<T> {
    fn process(&self, sample: Option<T>) -> T;
}

pub struct Block {
    next_block: Option<Box<Block>>,
}

impl Process<Sample<f32>> for Block {
    fn process(&self, _sample: Option<Sample<f32>>) -> Sample<f32> {
        let mut rng = rand::thread_rng();
        return rng.gen::<f32>();
    }
}

pub struct Graph {
    sample_rate: usize,
    entry_block: Block,
}

impl Graph {
    pub fn new(sample_rate: usize) -> Graph {
        return Graph {
            sample_rate,
            entry_block: Block {
                next_block: None,
            },
        };
    }
}

impl Graph {
    fn evaluate(&mut self) -> Sample<f32> {
        let mut current = &self.entry_block;
        let mut sample = current.process(None);
        loop {
            if let Some(next_block) = &current.next_block {
                current = next_block;
                sample = current.process(Some(sample));
            } else {
                break;
            }
        }
        return sample;
    }
}

impl Iterator for Graph {
    type Item = Sample<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.evaluate());
    }
}

impl Source for Graph {
    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate as u32;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}
