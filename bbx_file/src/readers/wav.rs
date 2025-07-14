use std::path::Path;

use bbx_dsp::{
    buffer::{AudioBuffer, Buffer},
    reader::Reader,
    sample::Sample,
};
use wavers::Wav;

pub struct WavFileReader<S: Sample> {
    channels: Vec<AudioBuffer<S>>,
    sample_rate: f64,
    num_channels: usize,
    num_samples: usize,
}

impl<S: Sample> WavFileReader<S> {
    pub fn from_path(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader: Wav<f32> = Wav::from_path(Path::new(file_path))?;

        let sample_rate = reader.sample_rate() as f64;
        let num_channels = reader.n_channels() as usize;
        let num_samples = reader.n_samples();

        let mut channels = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            channels.push(AudioBuffer::new(num_samples));
        }

        for (channel_index, channel) in reader.channels().enumerate() {
            for (sample_index, sample) in channel.iter().enumerate() {
                channels[channel_index][sample_index] = S::from_f64(*sample as f64);
            }
        }

        Ok(Self {
            channels,
            sample_rate,
            num_channels,
            num_samples,
        })
    }
}

impl<S: Sample> Reader<S> for WavFileReader<S> {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn num_samples(&self) -> usize {
        self.num_samples
    }

    fn read_channel(&self, channel_index: usize) -> &[S] {
        self.channels[channel_index].as_slice()
    }
}
