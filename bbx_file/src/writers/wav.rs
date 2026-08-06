//! WAV file writer via hound.

use std::{error::Error, fs::File, io::BufWriter, path::Path};

use bbx_core::Buffer;
use bbx_dsp::{buffer::SampleBuffer, context::DEFAULT_SAMPLE_RATE, sample::Sample, writer::Writer};
use hound::{SampleFormat, WavSpec, WavWriter};

const BIT_DEPTH: u16 = 32;

/// A WAV file writer implementing [`Writer`].
///
/// Writes 32-bit floating point WAV files. Call [`finalize`](Writer::finalize)
/// when done to ensure proper file closure.
pub struct WavFileWriter<S: Sample> {
    writer: Option<WavWriter<BufWriter<File>>>,
    sample_rate: f64,
    num_channels: usize,
    samples_written: usize,
    channel_buffers: Vec<SampleBuffer<S>>,
}

impl<S: Sample> WavFileWriter<S> {
    /// Create a `WavFileWriter` with the specified sample rate and
    /// number of audio channels.
    pub fn new(file_path: &str, sample_rate: f64, num_channels: usize) -> Result<Self, Box<dyn Error>> {
        let spec = WavSpec {
            channels: num_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: BIT_DEPTH,
            sample_format: SampleFormat::Float,
        };

        let writer = WavWriter::create(Path::new(file_path), spec)?;

        Ok(Self {
            writer: Some(writer),
            sample_rate,
            num_channels,
            samples_written: 0,
            channel_buffers: vec![SampleBuffer::new(DEFAULT_SAMPLE_RATE as usize); num_channels],
        })
    }
}

impl<S: Sample> Writer<S> for WavFileWriter<S> {
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn num_channels(&self) -> usize {
        self.num_channels
    }

    fn can_write(&self) -> bool {
        // NOTE: WAV files can generally always accept more data
        self.writer.is_some()
    }

    fn write_channel(&mut self, channel_index: usize, samples: &[S]) -> Result<(), Box<dyn Error>> {
        if channel_index >= self.num_channels {
            return Err("Channel index out of bounds".into());
        }

        self.channel_buffers[channel_index].extend_from_slice(samples);
        self.write_available_samples()?;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Box<dyn Error>> {
        self.write_available_samples()?;

        if let Some(writer) = self.writer.take() {
            writer.finalize()?;
        }

        Ok(())
    }
}

impl<S: Sample> WavFileWriter<S> {
    /// Write the available samples of each channel to the audio file.
    fn write_available_samples(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut writer) = self.writer {
            let min_len = self.channel_buffers.iter().map(|buf| buf.len()).min().unwrap_or(0);

            if min_len == 0 {
                return Ok(());
            }

            for sample_idx in 0..min_len {
                for channel_idx in 0..self.num_channels {
                    let sample = self.channel_buffers[channel_idx][sample_idx];
                    writer.write_sample(sample.to_f64() as f32)?;
                }
            }

            for channel_buffer in &mut self.channel_buffers {
                channel_buffer.drain_front(min_len);
            }

            self.samples_written += min_len * self.num_channels;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_wav_writer_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let writer = WavFileWriter::<f32>::new(path, 44100.0, 2);
        assert!(writer.is_ok());

        let writer = writer.unwrap();
        assert_eq!(writer.sample_rate(), 44100.0);
        assert_eq!(writer.num_channels(), 2);
        assert!(writer.can_write());
    }

    #[test]
    fn test_wav_writer_write_and_finalize() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = WavFileWriter::<f32>::new(path, 44100.0, 2).unwrap();

        // Write some samples
        let samples: Vec<f32> = (0..100).map(|i| (i as f32 / 100.0)).collect();
        writer.write_channel(0, &samples).unwrap();
        writer.write_channel(1, &samples).unwrap();
        writer.finalize().unwrap();

        // Verify file was created and has content
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_wav_writer_channel_bounds() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let mut writer = WavFileWriter::<f32>::new(path, 44100.0, 2).unwrap();

        // Writing to invalid channel should error
        let samples = vec![0.0f32; 10];
        let result = writer.write_channel(5, &samples);
        assert!(result.is_err());
    }
}
