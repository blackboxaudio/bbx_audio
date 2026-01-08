//! Sketch trait and registry system.

mod traits;

#[cfg(feature = "sketch-registry")]
mod registry;

#[cfg(feature = "sketch-registry")]
pub use registry::{SketchMetadata, SketchRegistry};
pub use traits::Sketch;
