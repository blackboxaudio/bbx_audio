use bbx_dsp::sample::Sample;

use crate::Source;

#[cfg(feature = "rodio")]
mod rodio;
#[cfg(feature = "rodio")]
pub use self::rodio::RodioBackend;

#[cfg(feature = "cpal")]
mod cpal;
#[cfg(feature = "cpal")]
pub use self::cpal::CpalBackend;

/// Adapter that converts any `Source<S>` to yield `f32` samples.
///
/// Used internally by backends that require f32 samples for output.
pub(crate) struct SourceAdapter<S: Sample> {
    source: Box<dyn Source<S>>,
}

impl<S: Sample> SourceAdapter<S> {
    pub fn new(source: Box<dyn Source<S>>) -> Self {
        Self { source }
    }

    pub fn channels(&self) -> u16 {
        self.source.channels()
    }

    pub fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }
}

impl<S: Sample> Iterator for SourceAdapter<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|s| s.to_f64() as f32)
    }
}
