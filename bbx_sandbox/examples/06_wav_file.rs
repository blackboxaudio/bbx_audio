use std::env;
use bbx_file::readers::wav::WavFileReader;

fn main() {
    let mut path = env::current_dir().unwrap().to_str().unwrap().to_owned();
    path.push_str("/examples/06_sample.wav");
    println!("PATH: {}", path);
    let mut reader = WavFileReader::open(path.as_str()).unwrap();
    let samples = reader.read_samples().unwrap();

    println!("Num Channels: {:?}", reader.metadata().num_channels);
    println!("Sample Rate: {:?}", reader.metadata().sample_rate);
    println!("Bit Depth: {:?}", reader.metadata().bits_per_sample);
    println!("Sample Format: {:?}", reader.metadata().format);
    println!("Samples: \n{:?}", samples);
}
