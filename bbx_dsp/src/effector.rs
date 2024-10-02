use crate::{
    context::Context,
    effectors::{flanger::FlangerEffector, mixer::MixerEffector, overdrive::OverdriveEffector},
    operation::Operation,
};

/// A type of DSP `Node` that produces an output signal by modifying an input signal.
pub enum Effector {
    Flanger(Context, f32, f32, f32, f32),
    Mixer,
    Overdrive,
}

impl Effector {
    /// Convert this `Effector` to an `Operation`, to store within a `Node` in a `Graph`.
    pub fn to_operation(self, _context: Context) -> Operation {
        match self {
            Effector::Flanger(context, depth, feedback, rate, delay_time) => {
                Box::new(FlangerEffector::new(context, depth, feedback, rate, delay_time))
            }
            Effector::Mixer => Box::new(MixerEffector),
            Effector::Overdrive => Box::new(OverdriveEffector),
        }
    }
}
