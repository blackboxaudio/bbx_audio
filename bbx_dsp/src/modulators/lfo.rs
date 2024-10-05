use bbx_buffer::buffer::AudioBuffer;

use crate::{
    context::Context,
    generators::wave_table::Waveform,
    modulator::ModulationDestination,
    process::{AudioInput, ModulationInput, Process},
};

const WAVE_TABLE_SIZE: usize = 128;

pub struct LowFrequencyOscillatorModulator {
    context: Context,
    wave_table: Vec<f32>,
    phase: f32,
    phase_increment: f32,
    waveform: Waveform,
}

impl LowFrequencyOscillatorModulator {
    pub fn new(context: Context, frequency: f32) -> Self {
        let wave_table = Self::create_wave_table(WAVE_TABLE_SIZE);
        let phase_increment = Self::calculate_phase_increment(context.sample_rate, frequency, wave_table.len());
        LowFrequencyOscillatorModulator {
            context,
            wave_table,
            phase: 0.0,
            phase_increment,
            waveform: Waveform::Sine,
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

impl LowFrequencyOscillatorModulator {
    pub fn get_frequency(&self) -> f32 {
        (self.phase_increment * self.context.sample_rate as f32) / self.wave_table.len() as f32
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment =
            Self::calculate_phase_increment(self.context.sample_rate, frequency, self.wave_table.len());
    }
}

impl LowFrequencyOscillatorModulator {
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

impl Process for LowFrequencyOscillatorModulator {
    fn process(
        &mut self,
        _inputs: &[AudioInput],
        _output: &mut [AudioBuffer<f32>],
        mod_inputs: &[ModulationInput],
        mod_output: &mut Vec<f32>,
    ) {
        let mut sample_idx: usize = 0;
        let freq_mod_input_idx = mod_inputs
            .iter()
            .position(|i| i.destination == ModulationDestination::Frequency);
        *mod_output = mod_output
            .iter_mut()
            .map(|_| {
                let sine_value = self.lerp();
                self.phase += self.phase_increment;
                self.phase %= self.wave_table.len() as f32;
                if let Some(freq_mod_input) = freq_mod_input_idx {
                    // TOOD: Change hard-coded value (aka depth)
                    let freq_mod = 55.0 * mod_inputs[freq_mod_input].as_slice()[sample_idx];
                    self.set_frequency(self.get_frequency() + freq_mod);
                    sample_idx += 1;
                }
                self.get_waveform_value(sine_value)
            })
            .collect();
    }
}
