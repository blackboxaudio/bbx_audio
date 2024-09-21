use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::convert::TryInto;

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
    file: File,
    metadata: WavMetadata,
}

impl WavFileReader {
    pub fn metadata(&self) -> WavMetadata {
        self.metadata
    }
}

impl WavFileReader {
    // Open the file and read its metadata
    pub fn open(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;

        // Read RIFF header
        let mut header = [0; 44];
        file.read_exact(&mut header)?;

        // Validate RIFF and WAVE tags
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid WAV file"));
        }

        // Read the fmt subchunk
        let audio_format = u16::from_le_bytes(header[20..22].try_into().unwrap());
        let num_channels = u16::from_le_bytes(header[22..24].try_into().unwrap());
        let sample_rate = u32::from_le_bytes(header[24..28].try_into().unwrap());
        let bits_per_sample = u16::from_le_bytes(header[34..36].try_into().unwrap());

        let format = match audio_format {
            1 => WavFormat::PCM,       // PCM (uncompressed)
            3 => WavFormat::Float,     // IEEE float
            _ => WavFormat::Unsupported,
        };

        let metadata = WavMetadata {
            sample_rate,
            num_channels,
            bits_per_sample,
            format,
        };

        // Find and skip to the data chunk
        file.seek(SeekFrom::Start(36))?;
        let mut chunk_header = [0; 8];
        loop {
            file.read_exact(&mut chunk_header)?;
            let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap());

            if &chunk_header[0..4] == b"data" {
                break; // We've found the "data" chunk
            } else {
                // Skip this chunk and go to the next one
                file.seek(SeekFrom::Current(chunk_size as i64))?;
            }
        }

        Ok(WavFileReader { file, metadata })
    }

    // Read the WAV file's audio data, normalizing to f32 format
    pub fn read_samples(&mut self) -> io::Result<Vec<f32>> {
        let data_chunk_size = {
            let mut chunk_size_buffer = [0; 4];
            self.file.read_exact(&mut chunk_size_buffer)?;
            u32::from_le_bytes(chunk_size_buffer)
        };

        let sample_count = data_chunk_size / (self.metadata.bits_per_sample as u32 / 8);
        let mut audio_data = Vec::with_capacity(sample_count as usize);
        println!("DATA: {:?}", audio_data);

        match self.metadata.format {
            WavFormat::PCM => match self.metadata.bits_per_sample {
                8 => {
                    let mut buffer = vec![0u8; sample_count as usize];
                    self.file.read_exact(&mut buffer)?;
                    for sample in buffer {
                        // 8-bit PCM is unsigned, normalize to [-1.0, 1.0]
                        audio_data.push((sample as f32 / 128.0) - 1.0);
                    }
                }
                16 => {
                    let mut buffer = vec![0u8; (sample_count * 2) as usize];
                    self.file.read_exact(&mut buffer)?;
                    for chunk in buffer.chunks_exact(2) {
                        let sample = i16::from_le_bytes(chunk.try_into().unwrap());
                        // Normalize to [-1.0, 1.0]
                        audio_data.push(sample as f32 / i16::MAX as f32);
                    }
                }
                24 => {
                    let mut buffer = vec![0u8; (sample_count * 3) as usize];
                    self.file.read_exact(&mut buffer)?;
                    for chunk in buffer.chunks_exact(3) {
                        // Manually convert 24-bit PCM to 32-bit signed integer
                        let sample = ((chunk[2] as i32) << 16)
                            | ((chunk[1] as i32) << 8)
                            | (chunk[0] as i32);
                        // Convert to [-1.0, 1.0]
                        audio_data.push(sample as f32 / 8_388_608.0); // 2^23
                    }
                }
                32 => {
                    let mut buffer = vec![0u8; (sample_count * 4) as usize];
                    self.file.read_exact(&mut buffer)?;
                    for chunk in buffer.chunks_exact(4) {
                        let sample = i32::from_le_bytes(chunk.try_into().unwrap());
                        // Normalize to [-1.0, 1.0]
                        audio_data.push(sample as f32 / i32::MAX as f32);
                    }
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported bit depth")),
            },
            WavFormat::Float => match self.metadata.bits_per_sample {
                32 => {
                    let mut buffer = vec![0u8; (sample_count * 4) as usize];
                    self.file.read_exact(&mut buffer)?;
                    for chunk in buffer.chunks_exact(4) {
                        let sample = f32::from_le_bytes(chunk.try_into().unwrap());
                        audio_data.push(sample);
                    }
                }
                64 => {
                    let mut buffer = vec![0u8; (sample_count * 8) as usize];
                    self.file.read_exact(&mut buffer)?;
                    for chunk in buffer.chunks_exact(8) {
                        let sample = f64::from_le_bytes(chunk.try_into().unwrap());
                        audio_data.push(sample as f32); // Convert to f32
                    }
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported bit depth for float")),
            },
            WavFormat::Unsupported => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported WAV format"))
            }
        }

        Ok(audio_data)
    }
}
