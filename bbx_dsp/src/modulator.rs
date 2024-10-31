use crate::{context::Context, modulators::lfo::LowFrequencyOscillatorModulator, operation::Operation};

pub enum Modulator {
    LowFrequencyOscillator { frequency: f32 },
}

impl Modulator {
    pub fn to_operation(self, context: Context) -> Operation {
        match self {
            Modulator::LowFrequencyOscillator { frequency } => {
                Box::new(LowFrequencyOscillatorModulator::new(context, frequency))
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ModulationDestination {
    Depth,
    Feedback,
    Frequency,
    Gain,
    Phase,
    Rate,
    Resonance,
    Time,
}
