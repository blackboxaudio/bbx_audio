#![cfg_attr(feature = "simd", feature(portable_simd))]

mod common;

use bbx_dsp::{
    block::Block,
    blocks::{
        effectors::{gain::GainBlock, panner::PannerBlock},
        generators::oscillator::OscillatorBlock,
        modulators::lfo::LfoBlock,
    },
    sample::Sample,
    waveform::Waveform,
};
use common::*;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

fn bench_oscillator_waveforms<S: Sample>(c: &mut Criterion, type_name: &str) {
    let waveforms = [
        ("sine", Waveform::Sine),
        ("sawtooth", Waveform::Sawtooth),
        ("square", Waveform::Square),
        ("triangle", Waveform::Triangle),
    ];

    let mut group = c.benchmark_group(format!("oscillator_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        for (waveform_name, waveform) in &waveforms {
            let bench_id = BenchmarkId::new(*waveform_name, buffer_size);

            group.bench_with_input(bench_id, buffer_size, |b, &size| {
                let context = create_context(size);
                let mut block = OscillatorBlock::<S>::new(S::from_f64(440.0), *waveform, None);
                let mut outputs = create_output_buffers::<S>(size, 1);
                let modulation_values: Vec<S> = vec![];

                b.iter(|| {
                    let inputs: Vec<&[S]> = vec![];
                    let mut output_slices = as_output_slices(&mut outputs);
                    block.process(
                        black_box(&inputs),
                        black_box(&mut output_slices),
                        black_box(&modulation_values),
                        black_box(&context),
                    );
                });
            });
        }
    }

    group.finish();
}

fn bench_oscillator_f32(c: &mut Criterion) {
    bench_oscillator_waveforms::<f32>(c, "f32");
}

fn bench_oscillator_f64(c: &mut Criterion) {
    bench_oscillator_waveforms::<f64>(c, "f64");
}

fn bench_panner<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("panner_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * 2));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = PannerBlock::<S>::new(S::from_f64(25.0));
            let inputs = create_input_buffers::<S>(size, 2);
            let mut outputs = create_output_buffers::<S>(size, 2);
            let modulation_values: Vec<S> = vec![];

            b.iter(|| {
                let input_slices = as_input_slices(&inputs);
                let mut output_slices = as_output_slices(&mut outputs);
                block.process(
                    black_box(&input_slices),
                    black_box(&mut output_slices),
                    black_box(&modulation_values),
                    black_box(&context),
                );
            });
        });
    }

    group.finish();
}

fn bench_panner_f32(c: &mut Criterion) {
    bench_panner::<f32>(c, "f32");
}

fn bench_panner_f64(c: &mut Criterion) {
    bench_panner::<f64>(c, "f64");
}

fn bench_gain<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("gain_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = GainBlock::<S>::new(S::from_f64(-6.0), Some(S::ONE));
            let inputs = create_input_buffers::<S>(size, 1);
            let mut outputs = create_output_buffers::<S>(size, 1);
            let modulation_values: Vec<S> = vec![];

            b.iter(|| {
                let input_slices = as_input_slices(&inputs);
                let mut output_slices = as_output_slices(&mut outputs);
                block.process(
                    black_box(&input_slices),
                    black_box(&mut output_slices),
                    black_box(&modulation_values),
                    black_box(&context),
                );
            });
        });
    }

    group.finish();
}

fn bench_gain_f32(c: &mut Criterion) {
    bench_gain::<f32>(c, "f32");
}

fn bench_gain_f64(c: &mut Criterion) {
    bench_gain::<f64>(c, "f64");
}

fn bench_lfo<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("lfo_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = LfoBlock::<S>::new(S::from_f64(5.0), S::from_f64(100.0), Waveform::Sine, None);
            let mut outputs = create_output_buffers::<S>(size, 1);
            let modulation_values: Vec<S> = vec![];

            b.iter(|| {
                let inputs: Vec<&[S]> = vec![];
                let mut output_slices = as_output_slices(&mut outputs);
                block.process(
                    black_box(&inputs),
                    black_box(&mut output_slices),
                    black_box(&modulation_values),
                    black_box(&context),
                );
            });
        });
    }

    group.finish();
}

fn bench_lfo_f32(c: &mut Criterion) {
    bench_lfo::<f32>(c, "f32");
}

fn bench_lfo_f64(c: &mut Criterion) {
    bench_lfo::<f64>(c, "f64");
}

criterion_group!(oscillator_benches, bench_oscillator_f32, bench_oscillator_f64,);

criterion_group!(panner_benches, bench_panner_f32, bench_panner_f64);

criterion_group!(gain_benches, bench_gain_f32, bench_gain_f64);

criterion_group!(lfo_benches, bench_lfo_f32, bench_lfo_f64);

criterion_main!(oscillator_benches, panner_benches, gain_benches, lfo_benches);
