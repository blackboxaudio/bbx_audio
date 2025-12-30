# FileOutputBlock

Write processed audio to files.

## Overview

`FileOutputBlock` wraps a `Writer` implementation to save audio from a DSP graph to disk.

## Creating a File Output

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::writers::wav::WavFileWriter;

let writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let file_out = builder.add_file_output(Box::new(writer));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Input | Left channel |
| 1 | Input | Right channel (if stereo) |
| N | Input | Channel N |

## Usage Examples

### Recording Synthesizer Output

```rust
let writer = WavFileWriter::new("synth_output.wav", 44100.0, 2)?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_gain(-6.0);
let file_out = builder.add_file_output(Box::new(writer));

builder.connect(osc, 0, gain, 0);
builder.connect(gain, 0, file_out, 0);

let mut graph = builder.build();

// Generate 5 seconds of audio
let samples_per_second = 44100;
let total_samples = samples_per_second * 5;
let buffer_size = 512;
let num_buffers = total_samples / buffer_size;

for _ in 0..num_buffers {
    let mut left = vec![0.0f32; buffer_size];
    let mut right = vec![0.0f32; buffer_size];
    let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
    graph.process_buffers(&mut outputs);
}

// Important: finalize to close the file
graph.finalize();
```

### Stereo Recording

```rust
let writer = WavFileWriter::new("stereo.wav", 44100.0, 2)?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let pan = builder.add_panner(0.25);  // Slightly right
let file_out = builder.add_file_output(Box::new(writer));

builder.connect(osc, 0, pan, 0);
builder.connect(pan, 0, file_out, 0);  // Left
builder.connect(pan, 1, file_out, 1);  // Right

let mut graph = builder.build();
// Process...
graph.finalize();
```

## Finalization

**Always call `finalize()`** after processing:

```rust
graph.finalize();
```

This:
- Flushes buffered data
- Updates file headers (WAV size fields)
- Closes the file handle

Without finalization, the file may be corrupt.

## Non-Blocking I/O

`FileOutputBlock` uses non-blocking I/O:

- Audio thread fills buffers
- Background thread writes to disk
- No blocking during process()
- Buffers are flushed during finalize()
