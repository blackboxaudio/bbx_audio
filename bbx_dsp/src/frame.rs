use crate::sample::Sample;

pub trait Frame: Copy + Clone + PartialEq {
    type Sample: Sample;
    type NumChannels: NumChannels;
    type Channels: Iterator<Item = Self::Sample>;

    const EQUILIBRIUM: Self;
    const NUM_CHANNELS: usize;

    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> Self::Sample;

    fn channel(&self, idx: usize) -> Option<&Self::Sample>;
    unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample;

    fn to_channels(self) -> Self::Channels;

    fn map<F, M>(self, map_fn: M) -> F
    where
        F: Frame,
        M: FnMut(Self::Sample) -> F::Sample;
}

pub trait NumChannels {}
pub struct NChannels<const N: usize> {}
impl<const N: usize> NumChannels for NChannels<N> {}

pub struct Channels<F: Frame> {
    next_idx: usize,
    frame: F,
}

impl<S: Sample, const N: usize> Frame for [S; N] {
    type Sample = S;
    type NumChannels = NChannels<N>;
    type Channels = Channels<Self>;

    const EQUILIBRIUM: Self =[S::EQUILIBRIUM; N];
    const NUM_CHANNELS: usize = N;

    #[inline]
    fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> Self::Sample
    {
        std::array::from_fn(f)
    }

    #[inline]
    fn channel(&self, idx: usize) -> Option<&Self::Sample> {
        self.get(idx)
    }

    #[inline(always)]
    unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample {
        self.get_unchecked(idx)
    }

    #[inline]
    fn to_channels(self) -> Self::Channels {
        Channels {
            next_idx: 0,
            frame: self,
        }
    }

    #[inline]
    fn map<F, M>(self, mut map_fn: M) -> F
    where
        F: Frame,
        M: FnMut(Self::Sample) -> F::Sample
    {
        F::from_fn(|channel_idx| {
            // Frames are guaranteed to have the same number of channels
            // because of the where clause
            unsafe { map_fn(*self.channel_unchecked(channel_idx)) }
        })
    }
}

impl<F> Iterator for Channels<F>
where
    F: Frame
{
    type Item = F::Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame.channel(self.next_idx).map(|&s| s).map(|s| {
            self.next_idx += 1;
            s
        })
    }
}
