//! Sketch trait definition.

use nannou::{App, Frame, event::Update};

/// Trait for nannou sketches that can be discovered and run.
///
/// Implementing this trait allows sketches to be registered and managed
/// by the `SketchRegistry`.
pub trait Sketch: Sized {
    /// The display name of this sketch.
    fn name(&self) -> &str;

    /// A brief description of what this sketch visualizes.
    fn description(&self) -> &str;

    /// Create the initial model/state for this sketch.
    fn model(app: &App) -> Self;

    /// Update the sketch state each frame.
    fn update(&mut self, app: &App, update: Update);

    /// Draw the sketch to the frame.
    fn view(&self, app: &App, frame: Frame);
}
