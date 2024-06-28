use std::fmt::{Display, Formatter};

use crate::{operation::Operation, process::Process, sample::Sample};

const WAVE_TABLE_SIZE: usize = 128;
const DEFAULT_GENERATOR_FREQUENCY: f32 = 110.0;

pub type Wavetable = Vec<Sample>;

/// A type of DSP `Block` that internally produces its own output signal.
pub struct Generator {
    sample_rate: usize,
    wave_table: Vec<Sample>,
    phase: f32,
    phase_increment: f32,
}

impl Generator {
    pub fn new(sample_rate: usize, frequency: Option<f32>) -> Generator {
        let wave_table = Self::create_wave_table(WAVE_TABLE_SIZE);
        let initial_frequency = if let Some(freq) = frequency {
            freq
        } else {
            DEFAULT_GENERATOR_FREQUENCY
        };
        let phase_increment = Self::calculate_phase_increment(sample_rate, initial_frequency, wave_table.len());
        return Generator {
            sample_rate,
            wave_table,
            phase: 0.0,
            phase_increment,
        };
    }

    pub fn to_operation(self) -> Operation {
        return Box::new(self);
    }

    fn create_wave_table(wave_table_size: usize) -> Wavetable {
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
        for n in 0..wave_table_size {
            let value = (n as f32 * std::f32::consts::PI * 2.0 / wave_table_size as f32).sin();
            wave_table.push(value);
        }
        return wave_table;
    }

    fn calculate_phase_increment(sample_rate: usize, frequency: f32, wave_table_length: usize) -> f32 {
        return frequency * wave_table_length as f32 / sample_rate as f32;
    }
}

impl Generator {
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment = Self::calculate_phase_increment(self.sample_rate, frequency, self.wave_table.len());
    }
}

impl Generator {
    fn lerp(&self) -> Sample {
        let truncated_index = self.phase as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        let next_index_weight = self.phase - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;
        return (self.wave_table[truncated_index] * truncated_index_weight)
            + (self.wave_table[next_index] * next_index_weight);
    }
}

impl Display for Generator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generator")
    }
}

impl Process for Generator {
    fn process(&mut self, _inputs: &Vec<Sample>) -> Sample {
        let sample = self.lerp();
        self.phase += self.phase_increment;
        self.phase %= self.wave_table.len() as f32;
        return sample;
    }
}
