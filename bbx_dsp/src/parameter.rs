//! Parameter modulation and smoothing system.
//!
//! This module provides the [`Parameter`] type, which combines a value source
//! (constant or modulated) with built-in smoothing for click-free parameter changes.

use std::marker::PhantomData;

use crate::{
    block::BlockId,
    sample::Sample,
    smoothing::{Linear, SmoothedValue, SmoothingStrategy},
};

/// Default ramp time in milliseconds for parameter smoothing.
pub const DEFAULT_RAMP_MS: f64 = 50.0;

/// Source of a parameter value: either constant or from a modulator block.
#[derive(Debug, Clone)]
pub enum ParameterSource<S: Sample> {
    /// A fixed value that doesn't change during processing.
    Constant(S),

    /// A value controlled by a modulator block.
    /// The [`BlockId`] references the source modulator.
    Modulated(BlockId),
}

impl<S: Sample> ParameterSource<S> {
    /// Get the value from this source.
    ///
    /// For modulated sources, safely looks up the modulation value
    /// and returns zero if the BlockId is out of bounds.
    #[inline]
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            ParameterSource::Constant(value) => *value,
            ParameterSource::Modulated(block_id) => modulation_values.get(block_id.0).copied().unwrap_or(S::ZERO),
        }
    }
}

/// A smoothed parameter with configurable interpolation strategy.
///
/// Combines a value source (constant or modulated) with built-in smoothing
/// to prevent audible clicks when parameters change.
///
/// # Type Parameters
///
/// - `S`: The sample type (f32 or f64)
/// - `T`: The smoothing strategy (defaults to [`Linear`])
///
/// # Examples
///
/// ```ignore
/// // Create a constant parameter with default linear smoothing
/// let gain = Parameter::<f32>::constant(0.5);
///
/// // Create a modulated parameter
/// let freq = Parameter::<f32>::modulated(lfo_id, 440.0);
///
/// // Create a parameter with custom ramp time
/// let pan = Parameter::<f32>::constant(0.0).with_ramp_ms(100.0);
/// ```
#[derive(Debug, Clone)]
pub struct Parameter<S: Sample, T: SmoothingStrategy = Linear> {
    source: ParameterSource<S>,
    smoother: SmoothedValue<S, T>,
    ramp_length_ms: f64,
    _marker: PhantomData<T>,
}

impl<S: Sample, T: SmoothingStrategy> Parameter<S, T> {
    /// Create a constant parameter with the given initial value.
    pub fn constant(value: S) -> Self {
        Self {
            source: ParameterSource::Constant(value),
            smoother: SmoothedValue::new(value),
            ramp_length_ms: DEFAULT_RAMP_MS,
            _marker: PhantomData,
        }
    }

    /// Create a modulated parameter.
    ///
    /// The initial value is used until the first modulation value arrives.
    pub fn modulated(block_id: BlockId, initial_value: S) -> Self {
        Self {
            source: ParameterSource::Modulated(block_id),
            smoother: SmoothedValue::new(initial_value),
            ramp_length_ms: DEFAULT_RAMP_MS,
            _marker: PhantomData,
        }
    }

    /// Set a custom ramp time in milliseconds.
    pub fn with_ramp_ms(mut self, ramp_ms: f64) -> Self {
        self.ramp_length_ms = ramp_ms;
        self
    }

    /// Prepare the parameter for audio processing with the given sample rate.
    ///
    /// Must be called before processing, typically during block initialization
    /// or when sample rate changes.
    pub fn prepare(&mut self, sample_rate: f64) {
        self.smoother.reset(sample_rate, self.ramp_length_ms);
    }

    /// Get the raw value from the source (without smoothing).
    #[inline]
    pub fn get_raw_value(&self, modulation_values: &[S]) -> S {
        self.source.get_value(modulation_values)
    }

    /// Update the smoother target from the current source value.
    ///
    /// Call once per buffer, before generating smoothed samples.
    #[inline]
    pub fn update_target(&mut self, modulation_values: &[S]) {
        let target = self.source.get_value(modulation_values);
        if (target.to_f64() - self.smoother.target().to_f64()).abs() > 1e-9 {
            self.smoother.set_target_value(target);
        }
    }

    /// Set the smoother target directly (for transformed values).
    ///
    /// Use this when you need to apply a transform before smoothing
    /// (e.g., dB to linear conversion for gain).
    #[inline]
    pub fn set_target(&mut self, value: S) {
        if (value.to_f64() - self.smoother.target().to_f64()).abs() > 1e-9 {
            self.smoother.set_target_value(value);
        }
    }

    /// Get the next smoothed value (call once per sample).
    #[inline]
    pub fn next_value(&mut self) -> S {
        self.smoother.get_next_value()
    }

    /// Check if the parameter is currently smoothing towards target.
    #[inline]
    pub fn is_smoothing(&self) -> bool {
        self.smoother.is_smoothing()
    }

    /// Get the current smoothed value without advancing.
    #[inline]
    pub fn current(&self) -> S {
        self.smoother.current()
    }

    /// Get the target value.
    #[inline]
    pub fn target(&self) -> S {
        self.smoother.target()
    }

    /// Skip ahead by N samples (useful for optimization).
    #[inline]
    pub fn skip(&mut self, num_samples: i32) {
        self.smoother.skip(num_samples);
    }

    /// Immediately set value without smoothing (for initialization).
    #[inline]
    pub fn set_immediate(&mut self, value: S) {
        self.smoother.set_immediate(value);
        if let ParameterSource::Constant(v) = &mut self.source {
            *v = value;
        }
    }

    /// Get a reference to the parameter source.
    #[inline]
    pub fn source(&self) -> &ParameterSource<S> {
        &self.source
    }

    /// Set the parameter source (for modulation setup).
    #[inline]
    pub fn set_source(&mut self, source: ParameterSource<S>) {
        self.source = source;
    }

    /// Fill a buffer with smoothed parameter values.
    ///
    /// This is the common pattern for applying smoothed parameters to audio:
    /// pre-compute all smoothed values into a stack buffer, then apply to samples.
    ///
    /// Returns `true` if smoothing was active (values varied), `false` if constant.
    #[inline]
    pub fn fill_buffer(&mut self, buffer: &mut [S], modulation_values: &[S]) -> bool {
        self.update_target(modulation_values);

        if !self.is_smoothing() {
            let value = self.current();
            buffer.iter_mut().for_each(|s| *s = value);
            return false;
        }

        for sample in buffer.iter_mut() {
            *sample = self.next_value();
        }
        true
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
