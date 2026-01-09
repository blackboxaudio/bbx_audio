# Benchmarking

Performance benchmarks for measuring SIMD optimization effectiveness and overall DSP performance.

## Overview

The `bbx_dsp` crate includes [Criterion](https://bheisler.github.io/criterion.rs/book/) benchmarks for:

- **Block micro-benchmarks** - Individual SIMD-optimized blocks in isolation
- **Graph integration benchmarks** - Realistic DSP graph configurations

Benchmarks support comparing **scalar vs SIMD** performance by running with and without the `simd` feature flag.

## Available Benchmark Suites

### simd_blocks

Micro-benchmarks for individual blocks:

| Block | What's measured | Variations |
|-------|-----------------|------------|
| OscillatorBlock | Waveform generation | sine, sawtooth, square, triangle |
| PannerBlock | Pan law + gain application | - |
| GainBlock | SIMD gain application | - |
| LfoBlock | Modulation signal generation | sine |

Each block is benchmarked with:
- Sample types: f32, f64
- Buffer sizes: 256, 512, 1024

### simd_graphs

Integration benchmarks for realistic DSP configurations:

| Graph | Blocks | Purpose |
|-------|--------|---------|
| simple_chain | Oscillator | Single-block baseline |
| effect_chain | Oscillator → Overdrive | Signal chain overhead |
| modulated_synth | Oscillator + LFO | Modulation path |
| multi_osc | 4 Oscillators | Multiple generator load |

## Running Benchmarks

### Basic Commands

```bash
# Run all benchmarks (scalar mode)
cargo bench -p bbx_dsp

# Run all benchmarks (SIMD mode, requires nightly)
cargo +nightly bench -p bbx_dsp --features simd

# Run specific benchmark suite
cargo bench -p bbx_dsp --bench simd_blocks
cargo bench -p bbx_dsp --bench simd_graphs

# Run specific benchmark by name filter
cargo bench -p bbx_dsp -- oscillator
cargo bench -p bbx_dsp -- "graph_simple"
```

### Comparing SIMD vs Scalar Performance

The recommended workflow for comparing performance:

```bash
# 1. Run scalar benchmarks and save as baseline
cargo bench --benches -p bbx_dsp -- --save-baseline scalar

# 2. Run SIMD benchmarks and compare against baseline
cargo +nightly bench --benches -p bbx_dsp --features simd -- --save-baseline scalar
```

This produces output showing the performance change:

```
oscillator_f32/sine/512
                        time:   [961.30 ns 962.33 ns 964.71 ns]
                        thrpt:  [530.73 Melem/s 532.04 Melem/s 532.61 Melem/s]
                 change:
                        time:   [-55.337% -53.509% -52.405%] (p = 0.00 < 0.05)
                        thrpt:  [+110.11% +115.10% +123.90%]
                        Performance has improved.
```

## Understanding Results

### Output Format

Criterion reports three values:
- **Lower bound** - Conservative estimate
- **Estimate** - Most likely value
- **Upper bound** - Optimistic estimate

### Throughput

Benchmarks report throughput in `Melem/s` (million elements per second), representing samples processed per second.

### HTML Reports

Criterion generates detailed HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` to view:

- Time distribution histograms
- Regression analysis
- Comparison charts between runs

## Benchmark Naming Convention

Benchmarks follow the pattern:

```
{category}_{sample_type}/{variant}/{buffer_size}
```

Examples:
- `oscillator_f32/sine/512` - f32 sine oscillator, 512 samples
- `panner_f64/1024` - f64 panner, 1024 samples
- `graph_simple_chain_f32/512` - Simple graph, f32, 512 samples

Use these names to filter benchmarks:

```bash
# All f32 benchmarks
cargo bench -p bbx_dsp -- f32

# All 512-sample benchmarks
cargo bench -p bbx_dsp -- /512

# All oscillator benchmarks
cargo bench -p bbx_dsp -- oscillator
```

## Adding New Benchmarks

### Block Benchmarks

Add to `bbx_dsp/benches/simd_blocks.rs`:

```rust
fn bench_my_block<S: Sample>(c: &mut Criterion, type_name: &str) {
    let mut group = c.benchmark_group(format!("my_block_{}", type_name));

    for buffer_size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(*buffer_size as u64));

        let bench_id = BenchmarkId::from_parameter(buffer_size);

        group.bench_with_input(bench_id, buffer_size, |b, &size| {
            let context = create_context(size);
            let mut block = MyBlock::<S>::new(/* params */);
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
```

### Graph Benchmarks

Add to `bbx_dsp/benches/simd_graphs.rs`:

```rust
fn create_my_graph<S: Sample>(buffer_size: usize) -> Graph<S> {
    let mut builder = GraphBuilder::new(SAMPLE_RATE, buffer_size, NUM_CHANNELS);
    // Add blocks and connections
    builder.build()
}

fn bench_my_graph_f32(c: &mut Criterion) {
    bench_graph::<f32, _>(c, "f32", "my_graph", create_my_graph);
}
```

## Tips

- **Warm cache**: Criterion automatically warms up before measuring
- **Stable environment**: Close other applications for consistent results
- **Multiple runs**: Run benchmarks multiple times to verify consistency
- **Release mode**: Benchmarks always run in release mode (`--release` is implicit)
