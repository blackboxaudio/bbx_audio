use crate::buffer::Buffer;
use crate::sample::Sample;

pub trait Frame {
    type Sample: Sample;
    type NumChannels: NumChannels;
    type Channels: Iterator<Item = Buffer<Self::Sample>>;

    const NUM_CHANNELS: usize;

    fn channel(&self, idx: usize) -> Option<&Buffer<Self::Sample>>;
    fn channels(self) -> Self::Channels;
}

pub trait NumChannels {}
pub struct NChannels<const N: usize> {}
impl<const N: usize> NumChannels for NChannels<N> {}
