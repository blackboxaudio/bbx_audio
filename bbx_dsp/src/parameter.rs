//! Parameter modulation system.
//!
//! This module provides the [`Parameter`] type, which allows block parameters
//! to be either constant values or modulated by other blocks (e.g., LFOs, envelopes).

use crate::{block::BlockId, sample::Sample};

/// A block parameter that can be constant or modulated.
///
/// Parameters allow block settings (like oscillator frequency or gain level)
/// to be controlled dynamically by modulator blocks during processing.
#[derive(Debug, Clone)]
pub enum Parameter<S: Sample> {
    /// A fixed value that doesn't change during processing.
    Constant(S),

    /// A value controlled by a modulator block.
    /// The [`BlockId`] references the source modulator.
    Modulated(BlockId),
}

impl<S: Sample> Parameter<S> {
    /// Get the appropriate value for a `Parameter`.
    ///
    /// For modulated parameters, safely looks up the modulation value
    /// and returns zero if the BlockId is out of bounds.
    #[inline]
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => {
                // Safe lookup to prevent panic in audio thread
                modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO)
            }
        }
    }
}

/// Describes a modulation output provided by a modulator block.
///
/// Modulator blocks (LFOs, envelopes) declare their outputs using this type,
/// specifying the output name and expected value range.
#[derive(Debug, Clone)]
pub struct ModulationOutput {
    /// Human-readable name for this output (e.g., "amplitude", "frequency").
    pub name: &'static str,

    /// Minimum value this output can produce.
    pub min_value: f64,

    /// Maximum value this output can produce.
    pub max_value: f64,
}
