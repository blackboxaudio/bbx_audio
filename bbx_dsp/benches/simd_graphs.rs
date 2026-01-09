#![cfg_attr(feature = "simd", feature(portable_simd))]

mod common;

use bbx_dsp::{graph::GraphBuilder, sample::Sample, waveform::Waveform};
use common::*;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

fn create_simple_chain<S: Sample>(buffer_size: usize) -> bbx_dsp::graph::Graph<S> {
    let mut builder = GraphBuilder::new(SAMPLE_RATE, buffer_size, NUM_CHANNELS);
    builder.add_oscillator(440.0, Waveform::Sine, None);
    builder.build()
}

fn create_effect_chain<S: Sample>(buffer_size: usize) -> bbx_dsp::graph::Graph<S> {
    let mut builder = GraphBuilder::new(SAMPLE_RATE, buffer_size, NUM_CHANNELS);
    let osc = builder.add_oscillator(440.0, Waveform::Sawtooth, None);
    let overdrive = builder.add_overdrive(2.0, 0.7, 0.5, SAMPLE_RATE);
    builder.connect(osc, 0, overdrive, 0);
    builder.build()
}

fn create_modulated_synth<S: Sample>(buffer_size: usize) -> bbx_dsp::graph::Graph<S> {
    let mut builder = GraphBuilder::new(SAMPLE_RATE, buffer_size, NUM_CHANNELS);
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
    let lfo = builder.add_lfo(5.0, 50.0, None);
    builder.modulate(lfo, osc, "frequency");
    builder.build()
}

fn create_multi_oscillator<S: Sample>(buffer_size: usize) -> bbx_dsp::graph::Graph<S> {
    let mut builder = GraphBuilder::new(SAMPLE_RATE, buffer_size, NUM_CHANNELS);
    builder.add_oscillator(220.0, Waveform::Sine, Some(1));
    builder.add_oscillator(440.0, Waveform::Sine, Some(2));
    builder.add_oscillator(660.0, Waveform::Sine, Some(3));
    builder.add_oscillator(880.0, Waveform::Sine, Some(4));
    builder.build()
}

fn bench_graph<S: Sample, F>(c: &mut Criterion, type_name: &str, graph_name: &str, graph_fn: F)
where
    F: Fn(usize) -> bbx_dsp::graph::Graph<S>,
{
    let mut group = c.benchmark_group(format!("graph_{}_{}", graph_name, type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64 * NUM_CHANNELS as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let mut graph = graph_fn(size);
            let mut left = vec![S::ZERO; size];
            let mut right = vec![S::ZERO; size];

            b.iter(|| {
                let mut output_buffers: Vec<&mut [S]> = vec![&mut left, &mut right];
                graph.process_buffers(black_box(&mut output_buffers));
            });
        });
    }

    group.finish();
}

fn bench_simple_chain_f32(c: &mut Criterion) {
    bench_graph::<f32, _>(c, "f32", "simple_chain", create_simple_chain);
}

fn bench_simple_chain_f64(c: &mut Criterion) {
    bench_graph::<f64, _>(c, "f64", "simple_chain", create_simple_chain);
}

fn bench_effect_chain_f32(c: &mut Criterion) {
    bench_graph::<f32, _>(c, "f32", "effect_chain", create_effect_chain);
}

fn bench_effect_chain_f64(c: &mut Criterion) {
    bench_graph::<f64, _>(c, "f64", "effect_chain", create_effect_chain);
}

fn bench_modulated_synth_f32(c: &mut Criterion) {
    bench_graph::<f32, _>(c, "f32", "modulated_synth", create_modulated_synth);
}

fn bench_modulated_synth_f64(c: &mut Criterion) {
    bench_graph::<f64, _>(c, "f64", "modulated_synth", create_modulated_synth);
}

fn bench_multi_osc_f32(c: &mut Criterion) {
    bench_graph::<f32, _>(c, "f32", "multi_osc", create_multi_oscillator);
}

fn bench_multi_osc_f64(c: &mut Criterion) {
    bench_graph::<f64, _>(c, "f64", "multi_osc", create_multi_oscillator);
}

criterion_group!(simple_chain_benches, bench_simple_chain_f32, bench_simple_chain_f64,);

criterion_group!(effect_chain_benches, bench_effect_chain_f32, bench_effect_chain_f64,);

criterion_group!(
    modulated_synth_benches,
    bench_modulated_synth_f32,
    bench_modulated_synth_f64,
);

criterion_group!(multi_osc_benches, bench_multi_osc_f32, bench_multi_osc_f64);

criterion_main!(
    simple_chain_benches,
    effect_chain_benches,
    modulated_synth_benches,
    multi_osc_benches
);
