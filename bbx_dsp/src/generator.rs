use crate::{generators::wavetable::WavetableGenerator, operation::Operation};

/// A type of DSP `Block` that internally produces its own output signal.
pub enum Generator {
    Wavetable { sample_rate: usize, frequency: f32 },
}

impl Generator {
    pub fn to_operation(self) -> Operation {
        return match self {
            Generator::Wavetable { sample_rate, frequency } => {
                Box::new(WavetableGenerator::new(sample_rate, frequency))
            }
        };
    }
}
