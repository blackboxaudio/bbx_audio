use std::env;

use bbx_file::{reader::Reader, readers::wav::WavFileReader};

fn main() {
    let mut path = env::current_dir().unwrap().to_str().unwrap().to_owned();
    path.push_str("/examples/06_sample.wav");
    println!("PATH: {}", path);
    let mut reader = WavFileReader::open(path.as_str());
    let samples = reader.read_file();
    println!("{:?}\n{}", samples, samples.len());
}
