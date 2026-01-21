//! WAV file reader via wavers.

use std::path::Path;

use bbx_core::Buffer;
use bbx_dsp::{buffer::SampleBuffer, reader::Reader, sample::Sample};
use wavers::Wav;

/// A WAV file reader implementing [`Reader`].
///
/// Loads the entire WAV file into memory on construction, providing
/// sample data via the `read_channel` method.
pub struct WavFileReader<S: Sample> {
    channel_buffers: Vec<SampleBuffer<S>>,
    sample_rate: f64,
    num_channels: usize,
    num_samples: usize,
}

impl<S: Sample> WavFileReader<S> {
    /// Create a `WavFileReader` for an audio file located
    /// at the specified file path.
    pub fn from_path(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader: Wav<f32> = Wav::from_path(Path::new(file_path))?;

        let sample_rate = reader.sample_rate() as f64;
        let num_channels = reader.n_channels() as usize;
        let num_samples = reader.n_samples();

        let mut channels = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            channels.push(SampleBuffer::new(num_samples));
        }

        for (channel_index, channel) in reader.channels().enumerate() {
            for (sample_index, sample) in channel.iter().enumerate() {
                channels[channel_index][sample_index] = S::from_f64(*sample as f64);
            }
        }

        Ok(Self {
            channel_buffers: channels,
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
        self.channel_buffers[channel_index].as_slice()
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use hound::{SampleFormat, WavSpec, WavWriter};
    use tempfile::NamedTempFile;

    use super::*;

    fn create_test_wav(sample_rate: u32, num_channels: u16, samples: &[Vec<f32>]) -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        let spec = WavSpec {
            channels: num_channels,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::new(BufWriter::new(temp_file.reopen().unwrap()), spec).unwrap();

        let num_samples = samples[0].len();
        for i in 0..num_samples {
            for channel in samples {
                writer.write_sample(channel[i]).unwrap();
            }
        }
        writer.finalize().unwrap();

        temp_file
    }

    #[test]
    fn test_wav_reader_mono_f32() {
        let samples = vec![vec![0.0, 0.5, 1.0, -0.5, -1.0]];
        let temp_file = create_test_wav(44100, 1, &samples);

        let reader = WavFileReader::<f32>::from_path(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(reader.num_channels(), 1);
        assert_eq!(reader.num_samples(), 5);
        assert_eq!(reader.sample_rate(), 44100.0);

        let channel = reader.read_channel(0);
        for (i, &expected) in samples[0].iter().enumerate() {
            assert!(
                (channel[i] - expected).abs() < 1e-6,
                "Sample {} mismatch: {} vs {}",
                i,
                channel[i],
                expected
            );
        }
    }

    #[test]
    fn test_wav_reader_mono_f64() {
        let samples = vec![vec![0.0, 0.25, 0.75, -0.25, -0.75]];
        let temp_file = create_test_wav(48000, 1, &samples);

        let reader = WavFileReader::<f64>::from_path(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(reader.num_channels(), 1);
        assert_eq!(reader.num_samples(), 5);
        assert_eq!(reader.sample_rate(), 48000.0);

        let channel = reader.read_channel(0);
        for (i, &expected) in samples[0].iter().enumerate() {
            assert!((channel[i] - expected as f64).abs() < 1e-6, "Sample {} mismatch", i);
        }
    }

    #[test]
    fn test_wav_reader_stereo_f32() {
        let left = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let right = vec![-0.1, -0.2, -0.3, -0.4, -0.5];
        let samples = vec![left.clone(), right.clone()];
        let temp_file = create_test_wav(44100, 2, &samples);

        let reader = WavFileReader::<f32>::from_path(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(reader.num_channels(), 2);
        // wavers.n_samples() returns total samples across all channels
        assert_eq!(reader.num_samples(), 10);

        let left_channel = reader.read_channel(0);
        let right_channel = reader.read_channel(1);

        for i in 0..5 {
            assert!((left_channel[i] - left[i]).abs() < 1e-6, "Left sample {} mismatch", i);
            assert!(
                (right_channel[i] - right[i]).abs() < 1e-6,
                "Right sample {} mismatch",
                i
            );
        }
    }

    #[test]
    fn test_wav_reader_sample_rate() {
        let samples = vec![vec![0.0; 10]];

        let temp_22050 = create_test_wav(22050, 1, &samples);
        let reader = WavFileReader::<f32>::from_path(temp_22050.path().to_str().unwrap()).unwrap();
        assert_eq!(reader.sample_rate(), 22050.0);

        let temp_96000 = create_test_wav(96000, 1, &samples);
        let reader = WavFileReader::<f32>::from_path(temp_96000.path().to_str().unwrap()).unwrap();
        assert_eq!(reader.sample_rate(), 96000.0);
    }

    #[test]
    fn test_wav_reader_invalid_path() {
        let result = WavFileReader::<f32>::from_path("/nonexistent/path/audio.wav");
        assert!(result.is_err());
    }
}
