//! DSP block implementations.
//!
//! Blocks are organized into categories:
//! - [`generators`]: Create audio signals (oscillators)
//! - [`effectors`]: Transform audio (gain, overdrive, panning)
//! - [`modulators`]: Generate control signals (LFOs, envelopes)
//! - [`io`]: Handle file and audio I/O

pub mod effectors;
pub mod generators;
pub mod io;
pub mod modulators;
