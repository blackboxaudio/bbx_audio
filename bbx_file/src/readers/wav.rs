use std::{
    fs::File,
};
use std::io::{BufReader, Read};
use hound::WavReader;
use crate::reader::Reader;

#[derive(Clone, Copy, Debug)]
pub enum WavFormat {
    PCM,
    Float,
    Unsupported,
}

#[derive(Clone, Copy, Debug)]
pub struct WavMetadata {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub bits_per_sample: u16,
    pub format: WavFormat,
}

pub struct WavFileReader {
    // file: File,
    // metadata: WavMetadata,
    reader: WavReader<BufReader<File>>,
}

impl Reader for WavFileReader {
    type Metadata = WavMetadata;

    // Open the file and read its metadata
    fn open(filename: &str) -> WavFileReader {
        let mut file = File::open(filename).unwrap();

        // Read RIFF header
        let mut header = [0; 44];
        file.read_exact(&mut header).unwrap();

        // Read the fmt subchunk
        let audio_format = u16::from_le_bytes(header[20..22].try_into().unwrap());
        let num_channels = u16::from_le_bytes(header[22..24].try_into().unwrap());
        let sample_rate = u32::from_le_bytes(header[24..28].try_into().unwrap());
        let bits_per_sample = u16::from_le_bytes(header[34..36].try_into().unwrap());

        let format = match audio_format {
            1 => WavFormat::PCM,   // PCM (uncompressed)
            3 => WavFormat::Float, // IEEE float
            _ => WavFormat::Unsupported,
        };

        let metadata = WavMetadata {
            sample_rate,
            num_channels,
            bits_per_sample,
            format,
        };
        println!("{:?}", metadata);

        let reader = WavReader::open(filename).unwrap();
        WavFileReader {
            reader,
        }
    }

    // Read the WAV file's audio data, normalizing to f32 format
    fn read_file(&mut self) -> Vec<f32> {
        self.reader.samples::<i16>().map(|s| s.unwrap() as f32 / i16::MAX as f32).collect()
    }
}
