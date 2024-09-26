use crate::{context::Context, generators::wave_table::WaveTableGenerator, operation::Operation};

/// A type of DSP `Block` that internally produces its own output signal.
pub enum Generator {
    WaveTable { frequency: f32 },
}

impl Generator {
    pub fn to_operation(self, context: Context) -> Operation {
        match self {
            Generator::WaveTable { frequency } => Box::new(WaveTableGenerator::new(context, frequency)),
        }
    }
}
