use crate::{buffer::AudioBuffer, sample::Sample};

pub trait Frame {
    type Sample: Sample;
    type NumChannels: NumChannels;
    type Channels: Iterator<Item = AudioBuffer<Self::Sample>>;

    const NUM_CHANNELS: usize;

    fn channel(&self, idx: usize) -> Option<&AudioBuffer<Self::Sample>>;
    fn channels(self) -> Self::Channels;
}

pub trait NumChannels {}
pub struct NChannels<const N: usize> {}
impl<const N: usize> NumChannels for NChannels<N> {}
