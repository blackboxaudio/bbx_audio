use std::path::Path;

use bbx_buffer::buffer::{AudioBuffer, Buffer};
use wavers::Wav;

use crate::reader::Reader;

pub struct WavFileReader {
    channels: Vec<AudioBuffer<f32>>,
}

impl WavFileReader {
    pub fn new(file_path: String) -> Self {
        let mut reader: Wav<f32> = Wav::from_path(Path::new(file_path.as_str())).unwrap();

        let _sample_rate = reader.sample_rate() as usize;
        let num_channels = reader.n_channels() as usize;
        let _num_samples = reader.n_samples();

        let mut channels = vec![AudioBuffer::new(reader.n_samples()); num_channels];
        for (channel_idx, channel) in reader.channels().enumerate() {
            for (sample_idx, sample) in channel.iter().enumerate() {
                channels[channel_idx][sample_idx] = *sample;
            }
        }

        WavFileReader { channels }
    }
}

impl Reader for WavFileReader {
    fn read_channel(&mut self, channel_idx: usize, sample_idx: usize, buffer_len: usize) -> &[f32] {
        &self.channels[channel_idx].as_slice()[sample_idx..(sample_idx + buffer_len)]
    }
}
