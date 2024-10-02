use crate::{
    context::Context,
    generators::{
        file_reader::FileReaderGenerator,
        wave_table::{WaveTableGenerator, Waveform},
    },
    operation::Operation,
};

/// A type of DSP `Node` that internally produces its own output signal.
pub enum Generator {
    FileReader { file_path: String },
    WaveTable { frequency: f32, waveform: Waveform },
}

impl Generator {
    /// Convert this `Effector` to an `Operation`, to store within a `Node` in a `Graph`.
    pub fn to_operation(self, context: Context) -> Operation {
        match self {
            Generator::FileReader { file_path } => Box::new(FileReaderGenerator::new(context, file_path)),
            Generator::WaveTable { frequency, waveform } => {
                Box::new(WaveTableGenerator::new(context, frequency, waveform))
            }
        }
    }
}
