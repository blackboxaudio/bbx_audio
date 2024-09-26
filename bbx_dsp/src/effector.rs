use crate::{
    effectors::{mixer::MixerEffector, overdrive::OverdriveEffector},
    operation::Operation,
};

/// A type of DSP `Block` that produces an output signal by modifying an input signal.
pub enum Effector {
    Mixer(),
    Overdrive(),
}

impl Effector {
    /// Convert this `Effector` to an `Operation`, to store within a `Block` in a `Graph`.
    pub fn to_operation(self) -> Operation {
        match self {
            Effector::Mixer() => Box::new(MixerEffector),
            Effector::Overdrive() => Box::new(OverdriveEffector),
        }
    }
}
