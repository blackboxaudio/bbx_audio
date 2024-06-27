use crate::{effectors::overdrive::OverdriveEffector, operation::Operation};

/// A type of DSP `Block` that produces an output signal by modifying an input signal.
pub enum Effector {
    Overdrive(),
}

impl Effector {
    pub fn to_operation(self) -> Operation {
        let effector = match self {
            Effector::Overdrive() => OverdriveEffector,
        };

        return Box::new(effector);
    }
}
