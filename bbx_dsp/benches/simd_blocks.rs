#![cfg_attr(feature = "simd", feature(portable_simd))]

mod common;

use bbx_dsp::{
    block::Block,
    blocks::{
        effectors::{
            channel_merger::ChannelMergerBlock,
            channel_splitter::ChannelSplitterBlock,
            dc_blocker::DcBlockerBlock,
            gain::GainBlock,
            low_pass_filter::LowPassFilterBlock,
            matrix_mixer::MatrixMixerBlock,
            mixer::{MixerBlock, NormalizationStrategy},
            overdrive::OverdriveBlock,
            panner::PannerBlock,
            vca::VcaBlock,
        },
        generators::oscillator::OscillatorBlock,
        modulators::{envelope::EnvelopeBlock, lfo::LfoBlock},
    },
    buffer::{AudioBuffer, Buffer},
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
                let mut block = OscillatorBlock::<S>::new(440.0, *waveform, None);
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
            let mut block = PannerBlock::<S>::new(25.0);
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
            let mut block = GainBlock::<S>::new(-6.0, Some(1.0));
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

fn bench_low_pass_filter<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("low_pass_filter_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = LowPassFilterBlock::<S>::new(1000.0, 0.707);
            block.prepare(&context);
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

fn bench_low_pass_filter_f32(c: &mut Criterion) {
    bench_low_pass_filter::<f32>(c, "f32");
}

fn bench_low_pass_filter_f64(c: &mut Criterion) {
    bench_low_pass_filter::<f64>(c, "f64");
}

fn bench_lfo<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("lfo_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = LfoBlock::<S>::new(5.0, 100.0, Waveform::Sine, None);
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

fn bench_mixer<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("mixer_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * 2)); // stereo output

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = MixerBlock::<S>::new(4, 2) // 4 stereo sources -> stereo out
                .with_normalization(NormalizationStrategy::Average);

            // 4 sources × 2 channels = 8 inputs
            let inputs = create_input_buffers::<S>(size, 8);
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

fn bench_mixer_f32(c: &mut Criterion) {
    bench_mixer::<f32>(c, "f32");
}

fn bench_mixer_f64(c: &mut Criterion) {
    bench_mixer::<f64>(c, "f64");
}

fn bench_matrix_mixer<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("matrix_mixer_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * 2)); // stereo output

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = MatrixMixerBlock::<S>::new(4, 2); // 4 inputs -> 2 outputs

            // Set up a simple mixing matrix
            block.set_gain(0, 0, S::from_f64(0.5));
            block.set_gain(1, 0, S::from_f64(0.5));
            block.set_gain(2, 1, S::from_f64(0.5));
            block.set_gain(3, 1, S::from_f64(0.5));

            let inputs = create_input_buffers::<S>(size, 4);
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

fn bench_matrix_mixer_f32(c: &mut Criterion) {
    bench_matrix_mixer::<f32>(c, "f32");
}

fn bench_matrix_mixer_f64(c: &mut Criterion) {
    bench_matrix_mixer::<f64>(c, "f64");
}

fn bench_channel_splitter<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("channel_splitter_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * 4)); // 4 channels

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let block = ChannelSplitterBlock::<S>::new(4);
            let mut block = block;

            let inputs = create_input_buffers::<S>(size, 4);
            let mut outputs = create_output_buffers::<S>(size, 4);
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

fn bench_channel_splitter_f32(c: &mut Criterion) {
    bench_channel_splitter::<f32>(c, "f32");
}

fn bench_channel_splitter_f64(c: &mut Criterion) {
    bench_channel_splitter::<f64>(c, "f64");
}

fn bench_channel_merger<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("channel_merger_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * 4)); // 4 channels

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let block = ChannelMergerBlock::<S>::new(4);
            let mut block = block;

            let inputs = create_input_buffers::<S>(size, 4);
            let mut outputs = create_output_buffers::<S>(size, 4);
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

fn bench_channel_merger_f32(c: &mut Criterion) {
    bench_channel_merger::<f32>(c, "f32");
}

fn bench_channel_merger_f64(c: &mut Criterion) {
    bench_channel_merger::<f64>(c, "f64");
}

fn bench_buffer_zeroize<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("buffer_zeroize_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let mut buffer = AudioBuffer::<S>::with_data(vec![S::ONE; size]);

            b.iter(|| {
                buffer.zeroize();
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_buffer_zeroize_f32(c: &mut Criterion) {
    bench_buffer_zeroize::<f32>(c, "f32");
}

fn bench_buffer_zeroize_f64(c: &mut Criterion) {
    bench_buffer_zeroize::<f64>(c, "f64");
}

fn bench_buffer_fill<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("buffer_fill_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let mut buffer = AudioBuffer::<S>::new(size);
            let value = S::from_f64(0.5);

            b.iter(|| {
                buffer.fill(value);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_buffer_fill_f32(c: &mut Criterion) {
    bench_buffer_fill::<f32>(c, "f32");
}

fn bench_buffer_fill_f64(c: &mut Criterion) {
    bench_buffer_fill::<f64>(c, "f64");
}

fn bench_vca<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("vca_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = VcaBlock::<S>::new();
            let audio_input = create_input_buffers::<S>(size, 1);
            let control_input: Vec<S> = (0..size).map(|_| S::from_f64(0.5)).collect();
            let mut outputs = create_output_buffers::<S>(size, 1);
            let modulation_values: Vec<S> = vec![];

            b.iter(|| {
                let inputs: Vec<&[S]> = vec![audio_input[0].as_slice(), control_input.as_slice()];
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

fn bench_vca_f32(c: &mut Criterion) {
    bench_vca::<f32>(c, "f32");
}

fn bench_vca_f64(c: &mut Criterion) {
    bench_vca::<f64>(c, "f64");
}

fn bench_dc_blocker<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("dc_blocker_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = DcBlockerBlock::<S>::new(true);
            block.set_sample_rate(SAMPLE_RATE);
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

fn bench_dc_blocker_f32(c: &mut Criterion) {
    bench_dc_blocker::<f32>(c, "f32");
}

fn bench_dc_blocker_f64(c: &mut Criterion) {
    bench_dc_blocker::<f64>(c, "f64");
}

fn bench_overdrive<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("overdrive_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = OverdriveBlock::<S>::new(2.0, 0.7, 0.5, SAMPLE_RATE);
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

fn bench_overdrive_f32(c: &mut Criterion) {
    bench_overdrive::<f32>(c, "f32");
}

fn bench_overdrive_f64(c: &mut Criterion) {
    bench_overdrive::<f64>(c, "f64");
}

fn bench_envelope<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("envelope_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = EnvelopeBlock::<S>::new(0.01, 0.1, 0.7, 0.2);
            block.note_on();
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

fn bench_envelope_f32(c: &mut Criterion) {
    bench_envelope::<f32>(c, "f32");
}

fn bench_envelope_f64(c: &mut Criterion) {
    bench_envelope::<f64>(c, "f64");
}

criterion_group!(oscillator_benches, bench_oscillator_f32, bench_oscillator_f64,);

criterion_group!(panner_benches, bench_panner_f32, bench_panner_f64);

criterion_group!(gain_benches, bench_gain_f32, bench_gain_f64);

criterion_group!(
    low_pass_filter_benches,
    bench_low_pass_filter_f32,
    bench_low_pass_filter_f64
);

criterion_group!(lfo_benches, bench_lfo_f32, bench_lfo_f64);

criterion_group!(mixer_benches, bench_mixer_f32, bench_mixer_f64);

criterion_group!(matrix_mixer_benches, bench_matrix_mixer_f32, bench_matrix_mixer_f64);

criterion_group!(
    channel_routing_benches,
    bench_channel_splitter_f32,
    bench_channel_splitter_f64,
    bench_channel_merger_f32,
    bench_channel_merger_f64
);

criterion_group!(
    buffer_benches,
    bench_buffer_zeroize_f32,
    bench_buffer_zeroize_f64,
    bench_buffer_fill_f32,
    bench_buffer_fill_f64
);

criterion_group!(vca_benches, bench_vca_f32, bench_vca_f64);

criterion_group!(dc_blocker_benches, bench_dc_blocker_f32, bench_dc_blocker_f64);

criterion_group!(overdrive_benches, bench_overdrive_f32, bench_overdrive_f64);

criterion_group!(envelope_benches, bench_envelope_f32, bench_envelope_f64);

criterion_main!(
    oscillator_benches,
    panner_benches,
    gain_benches,
    low_pass_filter_benches,
    lfo_benches,
    mixer_benches,
    matrix_mixer_benches,
    channel_routing_benches,
    buffer_benches,
    vca_benches,
    dc_blocker_benches,
    overdrive_benches,
    envelope_benches
);
