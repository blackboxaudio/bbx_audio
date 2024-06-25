use bbx_dsp::oscillator::Oscillator;
use rodio::{OutputStream, Source};

const SAMPLE_RATE: usize = 44100;
const OSC_FREQUENCY: f32 = 110.0;
const PLAYTIME_DURATION: u64 = 3;

pub fn run() {
    let wave_table_size: usize = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push((n as f32 * std::f32::consts::PI * 2.0 / wave_table_size as f32).sin());
    }
    let mut oscillator = Oscillator::new(SAMPLE_RATE, wave_table);
    oscillator.set_frequency(OSC_FREQUENCY);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _result = stream_handle.play_raw(oscillator.convert_samples());

    std::thread::sleep(std::time::Duration::from_secs(PLAYTIME_DURATION));
}
