use crate::readers::wav::WavFileReader;

pub trait Reader {
    type Metadata;

    fn open(filename: &str) -> WavFileReader;
    fn read_file(&mut self) -> Vec<f32>;
}
