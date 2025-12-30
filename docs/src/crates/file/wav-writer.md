# WAV Writer

Create WAV files from processed audio.

## Creating a Writer

```rust
use bbx_file::writers::wav::WavFileWriter;

let writer = WavFileWriter::<f32>::new(
    "output.wav",  // path
    44100.0,       // sample rate
    2,             // channels
)?;
```

## Writing Audio Data

### Per-Channel

```rust
use bbx_file::writers::wav::WavFileWriter;
use bbx_dsp::writer::Writer;

let mut writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;

writer.write_channel(0, &left_samples)?;
writer.write_channel(1, &right_samples)?;

writer.finalize()?;
```

### Interleaved

```rust
// Write interleaved samples [L, R, L, R, ...]
writer.write_interleaved(&interleaved_samples)?;
```

## Finalization

Always call `finalize()` to:
- Flush buffered data
- Update WAV header with correct sizes
- Close the file

```rust
writer.finalize()?;
```

Without finalization, the file may be corrupt or truncated.

## Writer Trait

`WavFileWriter` implements the `Writer` trait from bbx_dsp:

```rust
pub trait Writer<S: Sample>: Send {
    fn sample_rate(&self) -> f64;
    fn num_channels(&self) -> usize;
    fn write_channel(&mut self, channel: usize, samples: &[S]) -> Result<()>;
    fn finalize(&mut self) -> Result<()>;
}
```

## Usage with FileOutputBlock

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
use bbx_file::writers::wav::WavFileWriter;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Audio source
let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

// File output
let writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;
let file_out = builder.add_file_output(Box::new(writer));

builder.connect(osc, 0, file_out, 0);

let mut graph = builder.build();

// Process audio
for _ in 0..1000 {
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];
    let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
    graph.process_buffers(&mut outputs);
}

// Finalize
graph.finalize();
```

## Output Format

Default output format:
- IEEE Float 32-bit
- Little-endian
- Standard RIFF/WAVE header

## Error Handling

```rust
use bbx_file::writers::wav::WavFileWriter;

let writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2);

match writer {
    Ok(w) => {
        // Use writer
    }
    Err(e) => {
        eprintln!("Failed to create writer: {}", e);
    }
}
```

## Non-Blocking I/O

`FileOutputBlock` uses non-blocking I/O internally to avoid blocking the audio thread. Actual disk writes happen on a background thread.
