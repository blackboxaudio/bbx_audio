# Offline Renderer

Render DSP graphs to audio files at maximum CPU speed, bypassing real-time constraints.

## Overview

The `OfflineRenderer` processes a [`Graph`] directly to a [`Writer`] as fast as the CPU allows, making it ideal for:

- Bouncing/exporting audio to files
- Batch processing large amounts of audio
- Generating audio content faster than real-time
- Rendering complex DSP chains that would be too heavy for real-time playback

## Creating a Renderer

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::{OfflineRenderer, writers::wav::WavFileWriter};

// Build your DSP graph
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let osc = builder.add(OscillatorBlock::new(440.0, Waveform::Sine, None));
let graph = builder.build();

// Create a writer
let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;

// Create the offline renderer
let mut renderer = OfflineRenderer::new(graph, Box::new(writer));
```

## Rendering Audio

### By Duration

Render a specific number of seconds:

```rust
use bbx_file::RenderDuration;

let stats = renderer.render(RenderDuration::Duration(30))?;

println!("Rendered {}s in {:.2}s ({:.1}x realtime)",
    stats.duration_seconds,
    stats.render_time_seconds,
    stats.speedup);
```

### By Sample Count

Render a specific number of samples:

```rust
let stats = renderer.render(RenderDuration::Samples(44100 * 10))?;  // 10 seconds at 44.1kHz
```

## Render Statistics

`render()` returns `RenderStats` with performance information:

| Field | Type | Description |
|-------|------|-------------|
| `samples_rendered` | `u64` | Total samples rendered per channel |
| `duration_seconds` | `f64` | Audio duration in seconds |
| `render_time_seconds` | `f64` | Wall-clock time taken |
| `speedup` | `f64` | Speedup factor (duration / render_time) |

Speedup values greater than 1.0 indicate faster-than-realtime rendering.

## Complete Example

```rust
use bbx_dsp::{
    blocks::{GainBlock, LfoBlock, LowPassFilterBlock, OscillatorBlock, PannerBlock},
    graph::GraphBuilder,
    waveform::Waveform,
};
use bbx_file::{OfflineRenderer, RenderDuration, writers::wav::WavFileWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a DSP graph with modulated filter
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    let osc = builder.add(OscillatorBlock::new(220.0, Waveform::Sawtooth, None));
    let filter = builder.add(LowPassFilterBlock::new(800.0, 1.5));
    let lfo = builder.add(LfoBlock::new(0.5, 400.0, Waveform::Sine, None));
    let gain = builder.add(GainBlock::new(-6.0, None));
    let panner = builder.add(PannerBlock::new(0.0));

    builder.connect(osc, 0, filter, 0);
    builder.modulate(lfo, filter, "cutoff");
    builder.connect(filter, 0, gain, 0);
    builder.connect(gain, 0, panner, 0);

    let graph = builder.build();

    // Create writer and renderer
    let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;
    let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

    // Render 60 seconds of audio
    let stats = renderer.render(RenderDuration::Duration(60))?;

    println!("Rendered {:.1}s of audio in {:.2}s ({:.1}x realtime)",
        stats.duration_seconds,
        stats.render_time_seconds,
        stats.speedup);

    Ok(())
}
```

## Error Handling

```rust
use bbx_file::{OfflineRenderer, RenderDuration, RenderError};

match renderer.render(RenderDuration::Duration(30)) {
    Ok(stats) => {
        println!("Success! Speedup: {:.1}x", stats.speedup);
    }
    Err(RenderError::WriteFailed(e)) => {
        eprintln!("Write error: {}", e);
    }
    Err(RenderError::FinalizeFailed(e)) => {
        eprintln!("Failed to finalize: {}", e);
    }
    Err(RenderError::InvalidDuration(msg)) => {
        eprintln!("Invalid duration: {}", msg);
    }
}
```

## Constraints

- Writer sample rate must match graph sample rate
- Writer channel count must match graph channel count
- Duration must be positive (non-zero)

Mismatches cause a panic at renderer construction:

```rust
// This will panic - sample rates don't match
let graph = GraphBuilder::<f32>::new(44100.0, 512, 2).build();
let writer = WavFileWriter::new("out.wav", 48000.0, 2)?;  // 48kHz != 44.1kHz
let renderer = OfflineRenderer::new(graph, Box::new(writer));  // PANIC
```

## Recovering the Graph

After rendering, you can reclaim the graph for further processing:

```rust
let graph = renderer.into_graph();
```

Note: The writer is consumed during rendering and cannot be recovered.

## Comparison with FileOutputBlock

| Aspect | OfflineRenderer | FileOutputBlock |
|--------|-----------------|-----------------|
| Speed | Maximum CPU speed | Real-time constrained |
| Use case | Exporting/bouncing | Live recording |
| Graph ownership | Takes ownership | Part of graph |
| Thread model | Single-threaded render loop | Non-blocking I/O |

Use `OfflineRenderer` for batch export tasks. Use `FileOutputBlock` when recording audio during real-time playback.

## See Also

- [WAV Writer](wav-writer.md) - Creating WAV file writers
- [Working with Audio Files](../../tutorials/audio-files.md) - Complete tutorial
- Example `16_offline_rendering` - Complex unison synthesis demo
