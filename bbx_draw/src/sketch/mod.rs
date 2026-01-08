//! Sketch trait and registry system.

mod traits;

#[cfg(feature = "sketchbook")]
mod registry;

#[cfg(feature = "sketchbook")]
pub use registry::{SketchMetadata, SketchRegistry};
pub use traits::Sketch;
