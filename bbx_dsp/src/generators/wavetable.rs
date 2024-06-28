use std::fmt::{Display, Formatter};

use crate::{process::Process, sample::Sample};

const WAVE_TABLE_SIZE: usize = 128;

pub struct WavetableGenerator {
    sample_rate: usize,
    wave_table: Vec<Sample>,
    phase: f32,
    phase_increment: f32,
}

impl WavetableGenerator {
    pub fn new(sample_rate: usize, frequency: f32) -> WavetableGenerator {
        let wave_table = Self::create_wave_table(WAVE_TABLE_SIZE);
        let phase_increment = Self::calculate_phase_increment(sample_rate, frequency, wave_table.len());
        return WavetableGenerator {
            sample_rate,
            wave_table,
            phase: 0.0,
            phase_increment,
        };
    }

    fn create_wave_table(wave_table_size: usize) -> Vec<Sample> {
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

impl WavetableGenerator {
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment = Self::calculate_phase_increment(self.sample_rate, frequency, self.wave_table.len());
    }
}

impl WavetableGenerator {
    fn lerp(&self) -> Sample {
        let truncated_index = self.phase as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        let next_index_weight = self.phase - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;
        return (self.wave_table[truncated_index] * truncated_index_weight)
            + (self.wave_table[next_index] * next_index_weight);
    }
}

impl Display for WavetableGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "g_Wavetable")
    }
}

impl Process for WavetableGenerator {
    fn process(&mut self, _inputs: &Vec<Sample>) -> Sample {
        let sample = self.lerp();
        self.phase += self.phase_increment;
        self.phase %= self.wave_table.len() as f32;
        return sample;
    }
}
