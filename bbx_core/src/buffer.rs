//! Generic buffer trait for DSP operations.

/// A generic buffer interface for DSP operations.
///
/// Provides common operations for buffers used throughout the DSP system.
pub trait Buffer<T> {
    /// Get the length of the `Buffer`.
    fn len(&self) -> usize;

    /// Check if the `Buffer` is empty.
    fn is_empty(&self) -> bool;

    /// Get the `Buffer` as a data slice.
    fn as_slice(&self) -> &[T];

    /// Get the `Buffer` as a mutable data slice.
    fn as_mut_slice(&mut self) -> &mut [T];

    /// Remove all values from the `Buffer`
    fn clear(&mut self);

    /// Set all values in the `Buffer` to zero.
    fn zeroize(&mut self);
}
