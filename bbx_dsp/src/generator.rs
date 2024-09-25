use crate::{generators::wave_table::WaveTableGenerator, operation::Operation};

/// A type of DSP `Block` that internally produces its own output signal.
pub enum Generator {
    WaveTable { sample_rate: usize, frequency: f32 },
}

impl Generator {
    pub fn to_operation(self) -> Operation {
        match self {
            Generator::WaveTable { sample_rate, frequency } => {
                Box::new(WaveTableGenerator::new(sample_rate, frequency))
            }
        }
    }
}
