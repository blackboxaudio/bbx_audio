use crate::{
    buffer::{AudioBuffer, Buffer},
    context::Context,
    process::{AudioInput, Process},
};

const WAVE_TABLE_SIZE: usize = 128;

pub struct WaveTableGenerator {
    context: Context,
    wave_table: Vec<f32>,
    phase: f32,
    phase_increment: f32,
}

impl WaveTableGenerator {
    pub fn new(context: Context, frequency: f32) -> WaveTableGenerator {
        let wave_table = Self::create_wave_table(WAVE_TABLE_SIZE);
        let phase_increment = Self::calculate_phase_increment(context.sample_rate, frequency, wave_table.len());
        WaveTableGenerator {
            context,
            wave_table,
            phase: 0.0,
            phase_increment,
        }
    }

    fn create_wave_table(wave_table_size: usize) -> Vec<f32> {
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
        for n in 0..wave_table_size {
            let value = (n as f32 * std::f32::consts::PI * 2.0 / wave_table_size as f32).sin();
            wave_table.push(value);
        }
        wave_table
    }

    fn calculate_phase_increment(sample_rate: usize, frequency: f32, wave_table_length: usize) -> f32 {
        frequency * wave_table_length as f32 / sample_rate as f32
    }
}

impl WaveTableGenerator {
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment =
            Self::calculate_phase_increment(self.context.sample_rate, frequency, self.wave_table.len());
    }
}

impl WaveTableGenerator {
    fn lerp(&self) -> f32 {
        let truncated_index = self.phase as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        let next_index_weight = self.phase - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;
        (self.wave_table[truncated_index] * truncated_index_weight) + (self.wave_table[next_index] * next_index_weight)
    }
}

impl Process for WaveTableGenerator {
    fn process(&mut self, _inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        for output_buffer in output.iter_mut() {
            output_buffer.clear();
        }

        let mut output_iter = output.iter_mut();
        let model_buffer = output_iter.next().unwrap();
        model_buffer.apply_mut(|_| {
            let sample = self.lerp();
            self.phase += self.phase_increment;
            self.phase %= self.wave_table.len() as f32;
            sample
        });

        for channel_buffer in output_iter {
            channel_buffer.copy_from_slice(model_buffer.as_slice());
        }
    }
}
