use bbx_buffer::buffer::{AudioBuffer, Buffer};

use crate::{
    context::Context,
    process::{AudioInput, Process},
    utils::clear_output,
};

const WAVE_TABLE_SIZE: usize = 128;

pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct WaveTableGenerator {
    context: Context,
    wave_table: Vec<f32>,
    phase: f32,
    phase_increment: f32,
    waveform: Waveform,
}

impl WaveTableGenerator {
    pub fn new(context: Context, frequency: f32, waveform: Waveform) -> WaveTableGenerator {
        let wave_table = Self::create_wave_table(WAVE_TABLE_SIZE);
        let phase_increment = Self::calculate_phase_increment(context.sample_rate, frequency, wave_table.len());
        WaveTableGenerator {
            context,
            wave_table,
            phase: 0.0,
            phase_increment,
            waveform,
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

    fn get_waveform_value(&self, sine_value: f32) -> f32 {
        match self.waveform {
            Waveform::Sine => sine_value,
            Waveform::Square => {
                if sine_value >= 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => 2.0 * (sine_value - sine_value.floor()) - 1.0,
            Waveform::Triangle => 2.0 * sine_value.abs() - 1.0,
        }
    }
}

impl Process for WaveTableGenerator {
    fn process(&mut self, _inputs: &[AudioInput], output: &mut [AudioBuffer<f32>]) {
        clear_output(output);

        let mut output_iter = output.iter_mut();
        let model_buffer = output_iter.next().unwrap();
        model_buffer.apply_mut(|_| {
            let sine_value = self.lerp();
            self.phase += self.phase_increment;
            self.phase %= self.wave_table.len() as f32;
            self.get_waveform_value(sine_value)
        });

        for channel_buffer in output_iter {
            channel_buffer.copy_from_slice(model_buffer.as_slice());
        }
    }
}
