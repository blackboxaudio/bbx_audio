//! Offline audio renderer for DSP graphs.

use std::time::Instant;

use bbx_core::Buffer;
use bbx_dsp::{buffer::SampleBuffer, graph::Graph, sample::Sample, writer::Writer};

/// Specifies how long to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDuration {
    /// Render for a duration in seconds.
    Duration(usize),
    /// Render a specific number of samples (per channel).
    Samples(usize),
}

/// Error type for offline rendering operations.
#[derive(Debug)]
pub enum RenderError {
    /// The writer failed to accept samples.
    WriteFailed(Box<dyn std::error::Error>),
    /// The writer failed to finalize.
    FinalizeFailed(Box<dyn std::error::Error>),
    /// Invalid duration specified.
    InvalidDuration(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::WriteFailed(e) => write!(f, "write failed: {e}"),
            RenderError::FinalizeFailed(e) => write!(f, "finalize failed: {e}"),
            RenderError::InvalidDuration(msg) => write!(f, "invalid duration: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Statistics from a completed render.
#[derive(Debug, Clone)]
pub struct RenderStats {
    /// Total samples rendered (per channel).
    pub samples_rendered: u64,
    /// Total duration rendered in seconds.
    pub duration_seconds: f64,
    /// Wall-clock time taken in seconds.
    pub render_time_seconds: f64,
    /// Speedup factor (duration / render_time).
    pub speedup: f64,
}

/// Offline renderer for DSP graphs.
///
/// Renders a [`Graph`] to a [`Writer`] at maximum CPU speed, bypassing real-time
/// constraints. This is ideal for bouncing/exporting audio to files.
///
/// # Example
///
/// ```ignore
/// use bbx_dsp::graph::GraphBuilder;
/// use bbx_file::{OfflineRenderer, RenderDuration, writers::wav::WavFileWriter};
///
/// let graph = GraphBuilder::<f32>::new(44100.0, 512, 2)
///     .oscillator(440.0, Waveform::Sine)
///     .build();
///
/// let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;
/// let mut renderer = OfflineRenderer::new(graph, Box::new(writer));
/// let stats = renderer.render(RenderDuration::Duration(30))?;
/// println!("Rendered {}s in {:.2}s ({:.1}x realtime)",
///     stats.duration_seconds, stats.render_time_seconds, stats.speedup);
/// ```
pub struct OfflineRenderer<S: Sample> {
    graph: Graph<S>,
    writer: Box<dyn Writer<S>>,
    output_buffers: Vec<SampleBuffer<S>>,
    buffer_size: usize,
    sample_rate: f64,
    num_channels: usize,
}

impl<S: Sample> OfflineRenderer<S> {
    /// Create a new offline renderer.
    ///
    /// The graph should already be built via `GraphBuilder::build()`.
    /// The writer's sample rate and channel count should match the graph's context.
    ///
    /// # Arguments
    ///
    /// * `graph` - A prepared DSP graph (already built)
    /// * `writer` - A boxed writer implementation (e.g., WavFileWriter)
    ///
    /// # Panics
    ///
    /// Panics if the writer's sample rate or channel count doesn't match the graph.
    pub fn new(graph: Graph<S>, writer: Box<dyn Writer<S>>) -> Self {
        let context = graph.context();
        let buffer_size = context.buffer_size;
        let sample_rate = context.sample_rate;
        let num_channels = context.num_channels;

        assert!(
            (writer.sample_rate() - sample_rate).abs() < 1.0,
            "Writer sample rate ({}) must match graph sample rate ({})",
            writer.sample_rate(),
            sample_rate
        );
        assert_eq!(
            writer.num_channels(),
            num_channels,
            "Writer channel count ({}) must match graph channel count ({})",
            writer.num_channels(),
            num_channels
        );

        let output_buffers = (0..num_channels).map(|_| SampleBuffer::new(buffer_size)).collect();

        Self {
            graph,
            writer,
            output_buffers,
            buffer_size,
            sample_rate,
            num_channels,
        }
    }

    /// Render audio for the specified duration.
    ///
    /// Processes the graph at maximum CPU speed and writes output to the writer.
    /// Automatically calls `finalize()` on the writer when complete.
    ///
    /// # Arguments
    ///
    /// * `duration` - How long to render (in seconds or samples)
    ///
    /// # Returns
    ///
    /// Statistics about the render including speedup factor.
    pub fn render(&mut self, duration: RenderDuration) -> Result<RenderStats, RenderError> {
        let num_samples = match duration {
            RenderDuration::Duration(secs) => {
                if secs == 0 {
                    return Err(RenderError::InvalidDuration("Duration must be positive".to_string()));
                }
                (secs as f64 * self.sample_rate) as u64
            }
            RenderDuration::Samples(samples) => {
                if samples == 0 {
                    return Err(RenderError::InvalidDuration(
                        "Sample count must be positive".to_string(),
                    ));
                }
                samples as u64
            }
        };

        let start_time = Instant::now();
        let mut samples_rendered: u64 = 0;

        while samples_rendered < num_samples {
            let mut output_refs: Vec<&mut [S]> = self.output_buffers.iter_mut().map(|b| b.as_mut_slice()).collect();
            self.graph.process_buffers(&mut output_refs);

            let samples_remaining = num_samples - samples_rendered;
            let samples_to_write = (self.buffer_size as u64).min(samples_remaining) as usize;

            for (channel_idx, buffer) in self.output_buffers.iter().enumerate() {
                self.writer
                    .write_channel(channel_idx, &buffer.as_slice()[..samples_to_write])
                    .map_err(RenderError::WriteFailed)?;
            }

            samples_rendered += samples_to_write as u64;
        }

        self.writer.finalize().map_err(RenderError::FinalizeFailed)?;

        let render_time = start_time.elapsed().as_secs_f64();
        let duration_seconds = samples_rendered as f64 / self.sample_rate;

        Ok(RenderStats {
            samples_rendered,
            duration_seconds,
            render_time_seconds: render_time,
            speedup: duration_seconds / render_time,
        })
    }

    /// Get the sample rate of the renderer.
    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Get the number of channels.
    #[inline]
    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    /// Get the buffer size.
    #[inline]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Consume the renderer and return the graph.
    ///
    /// Useful if you need to reuse the graph after rendering.
    /// Note: The writer is consumed and cannot be retrieved.
    pub fn into_graph(self) -> Graph<S> {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use bbx_dsp::{blocks::OscillatorBlock, context::DEFAULT_SAMPLE_RATE, graph::GraphBuilder, waveform::Waveform};

    use super::*;

    struct TestWriter {
        sample_rate: f64,
        num_channels: usize,
        samples_written: Vec<Vec<f32>>,
        finalized: bool,
    }

    impl TestWriter {
        fn new(sample_rate: f64, num_channels: usize) -> Self {
            Self {
                sample_rate,
                num_channels,
                samples_written: vec![Vec::new(); num_channels],
                finalized: false,
            }
        }
    }

    impl Writer<f32> for TestWriter {
        fn sample_rate(&self) -> f64 {
            self.sample_rate
        }

        fn num_channels(&self) -> usize {
            self.num_channels
        }

        fn can_write(&self) -> bool {
            !self.finalized
        }

        fn write_channel(&mut self, channel_index: usize, samples: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
            self.samples_written[channel_index].extend_from_slice(samples);
            Ok(())
        }

        fn finalize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            self.finalized = true;
            Ok(())
        }
    }

    fn create_test_graph() -> Graph<f32> {
        let mut builder = GraphBuilder::<f32>::new(DEFAULT_SAMPLE_RATE, 512, 2);
        builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
        builder.build()
    }

    #[test]
    fn test_render_duration() {
        let graph = create_test_graph();
        let writer = TestWriter::new(DEFAULT_SAMPLE_RATE, 2);
        let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

        let stats = renderer.render(RenderDuration::Duration(1)).unwrap();

        assert_eq!(stats.samples_rendered, DEFAULT_SAMPLE_RATE as u64);
        assert!((stats.duration_seconds - 1.0).abs() < 0.01);
        assert!(stats.speedup > 1.0);
    }

    #[test]
    fn test_render_samples() {
        let graph = create_test_graph();
        let writer = TestWriter::new(DEFAULT_SAMPLE_RATE, 2);
        let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

        let stats = renderer.render(RenderDuration::Samples(1024)).unwrap();

        assert_eq!(stats.samples_rendered, 1024);
    }

    #[test]
    fn test_invalid_duration() {
        let graph = create_test_graph();
        let writer = TestWriter::new(DEFAULT_SAMPLE_RATE, 2);
        let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

        let result = renderer.render(RenderDuration::Duration(0));
        assert!(matches!(result, Err(RenderError::InvalidDuration(_))));
    }

    #[test]
    #[should_panic(expected = "sample rate")]
    fn test_mismatched_sample_rate() {
        let graph = create_test_graph();
        let writer = TestWriter::new(48000.0, 2);
        let _renderer = OfflineRenderer::new(graph, Box::new(writer));
    }

    #[test]
    #[should_panic(expected = "channel count")]
    fn test_mismatched_channels() {
        let graph = create_test_graph();
        let writer = TestWriter::new(DEFAULT_SAMPLE_RATE, 1);
        let _renderer = OfflineRenderer::new(graph, Box::new(writer));
    }
}
