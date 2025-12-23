use std::sync::atomic::{AtomicU32, Ordering};

use crate::{block::BlockId, sample::Sample};

/// Atomic f32 wrapper using bit-casting through u32.
///
/// This matches the memory layout of `std::atomic<float>` in C++
/// and allows lock-free parameter updates from the UI thread.
#[repr(C)]
pub struct AtomicF32(pub AtomicU32);

impl AtomicF32 {
    /// Create a new AtomicF32 with the given initial value.
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    /// Load the current value with relaxed ordering.
    #[inline]
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    /// Store a new value with relaxed ordering.
    #[inline]
    pub fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }
}

/// Types of parameters that DSP blocks can use.
///
/// Parameters can be:
/// - Constant: A fixed value set at graph construction
/// - Modulated: Driven by another block (e.g., LFO)
/// - External: Bound to an external atomic (e.g., JUCE AudioProcessorValueTreeState)
#[derive(Debug, Clone)]
pub enum Parameter<S: Sample> {
    /// A constant value.
    Constant(S),
    /// Modulated by another block's output.
    Modulated(BlockId),
    /// Bound to an external atomic source (for JUCE integration).
    /// The pointer must remain valid for the lifetime of the parameter.
    External(*const AtomicF32),
}

// Safety: The External variant only reads from the atomic, never writes.
// The atomic operations are inherently thread-safe.
unsafe impl<S: Sample> Send for Parameter<S> {}
unsafe impl<S: Sample> Sync for Parameter<S> {}

impl<S: Sample> Parameter<S> {
    /// Get the appropriate value for a `Parameter`.
    #[inline]
    pub fn get_value(&self, modulation_values: &[S]) -> S {
        match self {
            Parameter::Constant(value) => *value,
            Parameter::Modulated(block_id) => modulation_values[block_id.0],
            Parameter::External(ptr) => {
                if ptr.is_null() {
                    S::ZERO
                } else {
                    unsafe { S::from_f64((*(*ptr)).load() as f64) }
                }
            }
        }
    }

    /// Check if this parameter is externally bound.
    pub fn is_external(&self) -> bool {
        matches!(self, Parameter::External(_))
    }

    /// Bind this parameter to an external atomic source.
    ///
    /// # Safety
    /// The provided pointer must remain valid for the lifetime of the parameter.
    pub fn bind_external(&mut self, source: *const AtomicF32) {
        *self = Parameter::External(source);
    }

    /// Unbind from external source, reverting to a constant value.
    pub fn unbind_external(&mut self, default_value: S) {
        if self.is_external() {
            *self = Parameter::Constant(default_value);
        }
    }
}

/// Used for declaring outputs of a particular
/// `Modulator` block.
#[derive(Debug, Clone)]
pub struct ModulationOutput {
    pub name: &'static str,
    pub min_value: f64,
    pub max_value: f64,
}

/// A parameter that supports multiple modulation sources with depths.
///
/// This is the enhanced parameter type that supports:
/// - A base value (constant or externally bound)
/// - Up to N modulation sources with individual depths
///
/// The final value is computed as:
/// `base_value + sum(modulation_value[i] * depth[i])`
#[derive(Debug)]
pub struct ModulatableParam<S: Sample, const N: usize = 4> {
    /// Base value (used when no external source is bound)
    base_value: S,
    /// External atomic source (from JUCE)
    external_source: Option<*const AtomicF32>,
    /// Modulation slots: (source_block_id, depth)
    modulation_slots: [(Option<BlockId>, S); N],
}

// Safety: External pointer only read, atomic operations are thread-safe
unsafe impl<S: Sample, const N: usize> Send for ModulatableParam<S, N> {}
unsafe impl<S: Sample, const N: usize> Sync for ModulatableParam<S, N> {}

impl<S: Sample, const N: usize> ModulatableParam<S, N> {
    /// Create a new modulatable parameter with the given base value.
    pub fn new(value: S) -> Self {
        Self {
            base_value: value,
            external_source: None,
            modulation_slots: [(None, S::ZERO); N],
        }
    }

    /// Set the base value.
    pub fn set(&mut self, value: S) {
        self.base_value = value;
    }

    /// Get the base value (without modulation).
    pub fn get_base(&self) -> S {
        match self.external_source {
            Some(ptr) if !ptr.is_null() => unsafe { S::from_f64((*ptr).load() as f64) },
            _ => self.base_value,
        }
    }

    /// Bind to an external atomic source (JUCE parameter).
    ///
    /// # Safety
    /// The provided pointer must remain valid for the lifetime of the binding.
    pub fn bind_external(&mut self, source: *const AtomicF32) {
        self.external_source = Some(source);
    }

    /// Unbind from external source.
    pub fn unbind_external(&mut self) {
        self.external_source = None;
    }

    /// Check if externally bound.
    pub fn is_external(&self) -> bool {
        self.external_source.is_some()
    }

    /// Add a modulation source.
    ///
    /// Returns true if the source was added, false if all slots are full.
    pub fn add_modulation(&mut self, source: BlockId, depth: S) -> bool {
        for slot in &mut self.modulation_slots {
            if slot.0.is_none() {
                *slot = (Some(source), depth);
                return true;
            }
        }
        false
    }

    /// Remove a modulation source.
    ///
    /// Returns true if the source was found and removed.
    pub fn remove_modulation(&mut self, source: BlockId) -> bool {
        for slot in &mut self.modulation_slots {
            if slot.0 == Some(source) {
                *slot = (None, S::ZERO);
                return true;
            }
        }
        false
    }

    /// Set the depth of an existing modulation connection.
    pub fn set_modulation_depth(&mut self, source: BlockId, depth: S) {
        for slot in &mut self.modulation_slots {
            if slot.0 == Some(source) {
                slot.1 = depth;
                return;
            }
        }
    }

    /// Get the number of active modulation sources.
    pub fn modulation_count(&self) -> usize {
        self.modulation_slots.iter().filter(|s| s.0.is_some()).count()
    }

    /// Evaluate the parameter with all modulation applied.
    #[inline]
    pub fn evaluate(&self, modulation_values: &[S]) -> S {
        // Start with base value or external value
        let mut value = self.get_base();

        // Add modulation contributions
        for (source, depth) in &self.modulation_slots {
            if let Some(id) = source {
                if id.0 < modulation_values.len() {
                    value = value + modulation_values[id.0] * *depth;
                }
            }
        }

        value
    }
}

impl<S: Sample, const N: usize> Clone for ModulatableParam<S, N> {
    fn clone(&self) -> Self {
        Self {
            base_value: self.base_value,
            external_source: self.external_source,
            modulation_slots: self.modulation_slots,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_constant() {
        let param = Parameter::Constant(0.5f32);
        assert!((param.get_value(&[]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_modulated() {
        let param = Parameter::<f32>::Modulated(BlockId(0));
        let mod_values = vec![0.75f32, 0.0, 0.0];
        assert!((param.get_value(&mod_values) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_parameter_external() {
        let atomic = AtomicF32::new(0.25);
        let param = Parameter::<f32>::External(&atomic);
        assert!((param.get_value(&[]) - 0.25).abs() < 1e-6);

        atomic.store(0.8);
        assert!((param.get_value(&[]) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_modulatable_param() {
        let mut param = ModulatableParam::<f32, 4>::new(100.0);

        // Add two modulation sources
        assert!(param.add_modulation(BlockId(0), 10.0));
        assert!(param.add_modulation(BlockId(1), -5.0));
        assert_eq!(param.modulation_count(), 2);

        // Modulation values: [0.5, 1.0, ...]
        let mod_values = vec![0.5f32, 1.0, 0.0, 0.0];

        // Result: 100.0 + (0.5 * 10.0) + (1.0 * -5.0) = 100.0 + 5.0 - 5.0 = 100.0
        let result = param.evaluate(&mod_values);
        assert!((result - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_modulatable_param_external() {
        let atomic = AtomicF32::new(50.0);
        let mut param = ModulatableParam::<f32, 4>::new(100.0);

        // Bind to external
        param.bind_external(&atomic);
        assert!(param.is_external());

        // Base should now come from atomic
        assert!((param.get_base() - 50.0).abs() < 0.001);

        // Add modulation
        param.add_modulation(BlockId(0), 10.0);
        let result = param.evaluate(&[1.0]);
        // 50.0 + (1.0 * 10.0) = 60.0
        assert!((result - 60.0).abs() < 0.001);
    }
}
