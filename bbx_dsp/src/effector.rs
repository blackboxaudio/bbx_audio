use crate::{
    context::Context,
    effectors::{
        amplifier::AmplifierEffector, filter::FilterEffector, flanger::FlangerEffector, mixer::MixerEffector,
        overdrive::OverdriveEffector,
    },
    operation::Operation,
};

/// A type of DSP `Node` that produces an output signal by modifying an input signal.
pub enum Effector {
    /// A basic utility amplifier, requiring a gain value.
    Amplifier(f32),

    /// A low-pass filter, requiring a context, cutoff frequency, and resonance value.
    Filter(Context, f32, f32),

    /// A flanger, requiring a context, depth, feedback, rate, and delay time.
    Flanger(Context, f32, f32, f32, f32),

    /// A mixer, useful for summing multiple inputs.
    Mixer,

    /// An overdrive to add extra harmonics to a signal.
    Overdrive,
}

impl Effector {
    /// Convert this `Effector` to an `Operation`, to store within a `Node` in a `Graph`.
    pub fn to_operation(self, _context: Context) -> Operation {
        match self {
            Effector::Amplifier(gain) => Box::new(AmplifierEffector::new(gain)),
            Effector::Filter(context, cutoff, resonance) => Box::new(FilterEffector::new(context, cutoff, resonance)),
            Effector::Flanger(context, depth, feedback, rate, delay_time) => {
                Box::new(FlangerEffector::new(context, depth, feedback, rate, delay_time))
            }
            Effector::Mixer => Box::new(MixerEffector),
            Effector::Overdrive => Box::new(OverdriveEffector),
        }
    }
}
