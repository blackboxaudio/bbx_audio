use std::time::Duration;

use rodio::Source;

use crate::block::Block;
use crate::sample::{Sample};

type Wavetable = Vec<Sample<f32>>;

pub struct Oscillator {
    sample_rate: usize,
    wave_table: Wavetable,
    index: f32,
    index_increment: f32,
}

impl Oscillator {
    pub fn new(sample_rate: usize, wave_table: Wavetable) -> Oscillator {
        return Oscillator {
            sample_rate,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
        };
    }
}

impl Oscillator {
    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
    }
}

impl Oscillator {
    fn get_sample(&mut self) -> f32 {
        Self::process(self, None)
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;
        (self.wave_table[truncated_index] * truncated_index_weight) + (self.wave_table[next_index] * next_index_weight)
    }
}

impl Block<f32> for Oscillator {
    fn process(&mut self, _sample: Option<Sample<f32>>) -> Sample<f32> {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        sample as Sample<f32>
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}

impl Source for Oscillator {
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
