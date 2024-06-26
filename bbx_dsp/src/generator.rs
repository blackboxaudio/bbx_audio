use crate::{process::Process, sample::Sample};

const WAVE_TABLE_SIZE: usize = 128;

pub type Wavetable = Vec<Sample>;

/// A type of DSP `Block` that internally produces its own output signal.
pub struct Generator {
    sample_rate: usize,
    wave_table: Vec<Sample>,
    phase: f32,
    phase_increment: f32,
}

impl Generator {
    pub fn new(sample_rate: usize) -> Generator {
        return Generator {
            sample_rate,
            wave_table: Self::create_wave_table(WAVE_TABLE_SIZE),
            phase: 0.0,
            phase_increment: 0.0,
        };
    }

    fn create_wave_table(wave_table_size: usize) -> Wavetable {
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
        for n in 0..wave_table_size {
            let value = (n as f32 * std::f32::consts::PI * 2.0 / wave_table_size as f32).sin();
            wave_table.push(value);
        }
        return wave_table;
    }
}

impl Generator {
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
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

impl Process for Generator {
    fn process(&mut self, _sample: Option<Sample>) -> Sample {
        let sample = self.lerp();
        self.phase += self.phase_increment;
        self.phase %= self.wave_table.len() as f32;
        return sample
    }
}
