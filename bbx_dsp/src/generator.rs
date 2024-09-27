use crate::{
    context::Context,
    generators::wave_table::{WaveTableGenerator, Waveform},
    operation::Operation,
};

/// A type of DSP `Node` that internally produces its own output signal.
pub enum Generator {
    WaveTable { frequency: f32, waveform: Waveform },
}

impl Generator {
    /// Convert this `Effector` to an `Operation`, to store within a `Node` in a `Graph`.
    pub fn to_operation(self, context: Context) -> Operation {
        match self {
            Generator::WaveTable { frequency, waveform } => {
                Box::new(WaveTableGenerator::new(context, frequency, waveform))
            }
        }
    }
}
