#![allow(dead_code)]

use bbx_dsp::{context::DspContext, sample::Sample};

pub const BUFFER_SIZES: &[usize] = &[256, 512, 1024];
pub const SAMPLE_RATE: f64 = 44100.0;
pub const NUM_CHANNELS: usize = 2;

pub fn create_context(buffer_size: usize) -> DspContext {
    DspContext {
        sample_rate: SAMPLE_RATE,
        buffer_size,
        num_channels: NUM_CHANNELS,
        current_sample: 0,
    }
}

pub fn create_output_buffers<S: Sample>(buffer_size: usize, count: usize) -> Vec<Vec<S>> {
    (0..count).map(|_| vec![S::ZERO; buffer_size]).collect()
}

pub fn create_input_buffers<S: Sample>(buffer_size: usize, count: usize) -> Vec<Vec<S>> {
    (0..count)
        .map(|_| {
            (0..buffer_size)
                .map(|i| {
                    let phase = (i as f64 / buffer_size as f64) * f64::TAU;
                    S::from_f64(phase.sin())
                })
                .collect()
        })
        .collect()
}

pub fn as_input_slices<S>(buffers: &[Vec<S>]) -> Vec<&[S]> {
    buffers.iter().map(|v| v.as_slice()).collect()
}

pub fn as_output_slices<S>(buffers: &mut [Vec<S>]) -> Vec<&mut [S]> {
    buffers.iter_mut().map(|v| v.as_mut_slice()).collect()
}
