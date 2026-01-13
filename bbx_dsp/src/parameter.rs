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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_constant_f32() {
        let param = Parameter::Constant(42.0_f32);
        let modulation_values: Vec<f32> = vec![];
        let value = param.get_value(&modulation_values);
        assert!((value - 42.0).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_constant_f64() {
        let param = Parameter::Constant(42.0_f64);
        let modulation_values: Vec<f64> = vec![];
        let value = param.get_value(&modulation_values);
        assert!((value - 42.0).abs() < 1e-12);
    }

    #[test]
    fn test_parameter_modulated_valid_index_f32() {
        let param = Parameter::Modulated(BlockId(1));
        let modulation_values: Vec<f32> = vec![10.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!((value - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_modulated_valid_index_f64() {
        let param = Parameter::Modulated(BlockId(1));
        let modulation_values: Vec<f64> = vec![10.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!((value - 20.0).abs() < 1e-12);
    }

    #[test]
    fn test_parameter_modulated_out_of_bounds_returns_zero_f32() {
        let param = Parameter::Modulated(BlockId(10));
        let modulation_values: Vec<f32> = vec![10.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!(value.abs() < 1e-10, "Out of bounds should return zero");
    }

    #[test]
    fn test_parameter_modulated_out_of_bounds_returns_zero_f64() {
        let param = Parameter::Modulated(BlockId(10));
        let modulation_values: Vec<f64> = vec![10.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!(value.abs() < 1e-15, "Out of bounds should return zero");
    }

    #[test]
    fn test_parameter_modulated_empty_array_returns_zero_f32() {
        let param = Parameter::Modulated(BlockId(0));
        let modulation_values: Vec<f32> = vec![];
        let value = param.get_value(&modulation_values);
        assert!(value.abs() < 1e-10, "Empty array should return zero");
    }

    #[test]
    fn test_parameter_modulated_empty_array_returns_zero_f64() {
        let param = Parameter::Modulated(BlockId(0));
        let modulation_values: Vec<f64> = vec![];
        let value = param.get_value(&modulation_values);
        assert!(value.abs() < 1e-15, "Empty array should return zero");
    }

    #[test]
    fn test_parameter_modulated_index_zero_f32() {
        let param = Parameter::Modulated(BlockId(0));
        let modulation_values: Vec<f32> = vec![99.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!((value - 99.0).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_modulated_index_zero_f64() {
        let param = Parameter::Modulated(BlockId(0));
        let modulation_values: Vec<f64> = vec![99.0, 20.0, 30.0];
        let value = param.get_value(&modulation_values);
        assert!((value - 99.0).abs() < 1e-12);
    }

    #[test]
    fn test_parameter_constant_negative_value_f32() {
        let param = Parameter::Constant(-42.0_f32);
        let modulation_values: Vec<f32> = vec![];
        let value = param.get_value(&modulation_values);
        assert!((value - (-42.0)).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_constant_zero_f32() {
        let param = Parameter::Constant(0.0_f32);
        let modulation_values: Vec<f32> = vec![];
        let value = param.get_value(&modulation_values);
        assert!(value.abs() < 1e-10);
    }

    #[test]
    fn test_parameter_modulated_negative_value_f32() {
        let param = Parameter::Modulated(BlockId(0));
        let modulation_values: Vec<f32> = vec![-50.0];
        let value = param.get_value(&modulation_values);
        assert!((value - (-50.0)).abs() < 1e-6);
    }

    #[test]
    fn test_modulation_output_creation() {
        let output = ModulationOutput {
            name: "test",
            min_value: -1.0,
            max_value: 1.0,
        };
        assert_eq!(output.name, "test");
        assert!((output.min_value - (-1.0)).abs() < 1e-10);
        assert!((output.max_value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_clone_f32() {
        let param1 = Parameter::Constant(42.0_f32);
        let param2 = param1.clone();

        let modulation_values: Vec<f32> = vec![];
        let value1 = param1.get_value(&modulation_values);
        let value2 = param2.get_value(&modulation_values);

        assert!((value1 - value2).abs() < 1e-10);
    }

    #[test]
    fn test_parameter_modulated_clone_f32() {
        let param1 = Parameter::Modulated::<f32>(BlockId(1));
        let param2 = param1.clone();

        let modulation_values: Vec<f32> = vec![10.0, 20.0];
        let value1 = param1.get_value(&modulation_values);
        let value2 = param2.get_value(&modulation_values);

        assert!((value1 - value2).abs() < 1e-10);
    }
}
