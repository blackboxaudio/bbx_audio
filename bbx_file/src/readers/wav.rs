use std::path::Path;

use bbx_buffer::buffer::{AudioBuffer, Buffer};
use wavers::Wav;

use crate::reader::Reader;

pub struct WavFileReader {
    sample_rate: usize,
    num_channels: usize,
    num_samples: usize,
    channels: Vec<AudioBuffer<f32>>,
}

impl WavFileReader {
    pub fn new(file_path: String) -> Self {
        let mut reader: Wav<f32> = Wav::from_path(Path::new(file_path.as_str())).unwrap();

        let sample_rate = reader.sample_rate() as usize;
        let num_channels = reader.n_channels() as usize;
        let num_samples = reader.n_samples();

        println!("{} {} {}", sample_rate, num_channels, num_samples);

        let mut channels = vec![AudioBuffer::new(reader.n_samples()); num_channels];
        for (channel_idx, channel) in reader.channels().enumerate() {
            for (sample_idx, sample) in channel.iter().enumerate() {
                channels[channel_idx][sample_idx] = *sample;
            }
        }

        WavFileReader {
            sample_rate,
            num_channels,
            num_samples,
            channels,
        }
    }

    pub fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    pub fn num_samples(&self) -> usize {
        self.num_samples
    }
}

impl Reader for WavFileReader {
    fn read_channel(&mut self, channel_idx: usize, sample_idx: usize, buffer_len: usize) -> &[f32] {
        &self.channels[channel_idx].as_slice()[sample_idx..(sample_idx + buffer_len)]
    }
    fn read_sample(&self, channel_idx: usize, sample_idx: usize) -> f32 {
        self.channels[channel_idx][sample_idx]
    }
}
