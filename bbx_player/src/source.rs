use bbx_dsp::sample::Sample;

/// A source of audio samples.
///
/// This trait extends `Iterator` with audio-specific metadata (channels and sample rate).
/// Implement this trait for custom audio sources that you want to play through
/// [`play_source`](crate::play_source).
///
/// # Examples
///
/// ```ignore
/// use bbx_player::Source;
///
/// struct MySynth {
///     sample_rate: u32,
///     phase: f32,
/// }
///
/// impl Iterator for MySynth {
///     type Item = f32;
///
///     fn next(&mut self) -> Option<Self::Item> {
///         let sample = (self.phase * std::f32::consts::TAU).sin();
///         self.phase += 440.0 / self.sample_rate as f32;
///         if self.phase >= 1.0 { self.phase -= 1.0; }
///         Some(sample)
///     }
/// }
///
/// impl Source<f32> for MySynth {
///     fn channels(&self) -> u16 { 1 }
///     fn sample_rate(&self) -> u32 { self.sample_rate }
/// }
/// ```
pub trait Source<S: Sample>: Iterator<Item = S> + Send + 'static {
    /// Returns the number of audio channels.
    fn channels(&self) -> u16;

    /// Returns the sample rate in Hz.
    fn sample_rate(&self) -> u32;
}
