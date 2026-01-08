//! Sketch trait and sketchbook system.

mod traits;

#[cfg(feature = "sketchbook")]
mod sketchbook;

#[cfg(feature = "sketchbook")]
pub use sketchbook::{SketchMetadata, Sketchbook};
pub use traits::Sketch;
