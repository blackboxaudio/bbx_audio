# bbx_file

Audio file I/O implementations for the `bbx_dsp` crate.

## Features

- **WAV reading**: Load WAV files via `wavers`
- **WAV writing**: Create 32-bit float WAV files via `hound`
- **Offline rendering**: Render DSP graphs to files faster than realtime
- **Trait implementations**: Implements `Reader` and `Writer` from `bbx_dsp`

## Supported Formats

| Format | Read | Write |
|--------|------|-------|
| WAV    | Yes  | Yes   |

## Usage

### Reading WAV Files

```rust
use bbx_file::readers::wav::WavFileReader;
use bbx_dsp::reader::Reader;

let reader = WavFileReader::<f32>::from_path("audio.wav")?;

println!("Sample rate: {}", reader.sample_rate());
println!("Channels: {}", reader.num_channels());
println!("Samples: {}", reader.num_samples());

// Access channel data
let left_channel = reader.read_channel(0);
```

### Writing WAV Files

```rust
use bbx_file::writers::wav::WavFileWriter;
use bbx_dsp::writer::Writer;

let mut writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;

// Write samples per channel
writer.write_channel(0, &left_samples)?;
writer.write_channel(1, &right_samples)?;

// Finalize to close the file properly
writer.finalize()?;
```

### Integration with bbx_dsp

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("input.wav")?;
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let file_in = builder.add_file_input(Box::new(reader));
```

### Offline Rendering

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::{OfflineRenderer, RenderDuration, writers::wav::WavFileWriter};

let graph = GraphBuilder::<f32>::new(44100.0, 512, 2)
    // ... add blocks and connections
    .build();

let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;
let mut renderer = OfflineRenderer::new(graph, Box::new(writer));

let stats = renderer.render(RenderDuration::Duration(30))?;
println!("Rendered {}s in {:.2}s ({:.1}x realtime)",
    stats.duration_seconds, stats.render_time_seconds, stats.speedup);
```

## License

MIT
