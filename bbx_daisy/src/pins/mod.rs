//! Pin mappings for Daisy hardware variants.
//!
//! Each board variant has its own pin mapping module that defines
//! the GPIO assignments for audio, peripherals, and user I/O.

#![allow(unused_imports)]

#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
pub mod seed;

#[cfg(feature = "pod")]
pub mod pod;

#[cfg(feature = "patch_sm")]
pub mod patch_sm;

#[cfg(feature = "patch_sm")]
pub use patch_sm::*;
#[cfg(feature = "pod")]
pub use pod::*;
#[cfg(any(feature = "seed", feature = "seed_1_1", feature = "seed_1_2"))]
pub use seed::*;
