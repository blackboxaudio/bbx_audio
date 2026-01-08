//! Audio visualization primitives for nannou sketches.
//!
//! `bbx_draw` provides visualizers that can be embedded in any nannou application,
//! with lock-free communication between audio and visualization threads via
//! `bbx_core::SpscRingBuffer`.
//!
//! # Visualizers
//!
//! - [`GraphTopologyVisualizer`](visualizers::GraphTopologyVisualizer) - DSP graph topology display
//! - [`WaveformVisualizer`](visualizers::WaveformVisualizer) - Oscilloscope-style waveform
//! - [`SpectrumAnalyzer`](visualizers::SpectrumAnalyzer) - FFT-based spectrum display
//! - [`MidiActivityVisualizer`](visualizers::MidiActivityVisualizer) - MIDI note activity
//!
//! # Example
//!
//! ```ignore
//! use bbx_draw::{Visualizer, visualizers::GraphTopologyVisualizer};
//! use bbx_dsp::graph::GraphBuilder;
//!
//! // Capture topology from graph builder
//! let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
//! builder.add_oscillator(440.0, Waveform::Sine, None);
//! let topology = builder.capture_topology();
//!
//! // Create visualizer
//! let visualizer = GraphTopologyVisualizer::new(topology);
//! ```

pub mod bridge;
pub mod color;
pub mod config;
pub mod sketch;
pub mod visualizers;

pub use bridge::{
    AudioBridgeConsumer, AudioBridgeProducer, AudioFrame, MidiBridgeConsumer, MidiBridgeProducer, audio_bridge,
    midi_bridge,
};
use nannou::geom::Rect;
pub use visualizers::{GraphTopologyVisualizer, MidiActivityVisualizer, SpectrumAnalyzer, WaveformVisualizer};

/// Core trait for all visualizers.
///
/// Visualizers follow a two-phase update/draw pattern compatible with nannou's
/// model-update-view architecture.
pub trait Visualizer {
    /// Update internal state (called each frame before drawing).
    ///
    /// This is where visualizers should consume data from their SPSC bridges
    /// and update any internal buffers or state.
    fn update(&mut self);

    /// Draw the visualization within the given bounds.
    ///
    /// The `bounds` rectangle defines the area where the visualizer should render.
    /// This allows multiple visualizers to be arranged in a layout.
    fn draw(&self, draw: &nannou::Draw, bounds: Rect);
}
