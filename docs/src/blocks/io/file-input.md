# FileInputBlock

Read audio from files for processing in DSP graphs.

## Overview

`FileInputBlock` wraps a `Reader` implementation to provide file-based audio input to a DSP graph.

## Creating a File Input

```rust
use bbx_dsp::{blocks::FileInputBlock, graph::GraphBuilder};
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("input.wav")?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let file_in = builder.add(FileInputBlock::new(Box::new(reader)));
```

## Port Layout

| Port | Direction | Description |
|------|-----------|-------------|
| 0 | Output | Left channel |
| 1 | Output | Right channel (if stereo) |
| N | Output | Channel N |

## Usage Examples

### Basic File Playback

```rust
use bbx_dsp::{blocks::{FileInputBlock, GainBlock}, graph::GraphBuilder};

let reader = WavFileReader::from_path("audio.wav")?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let file_in = builder.add(FileInputBlock::new(Box::new(reader)));

// Process through effects
let gain = builder.add(GainBlock::new(-6.0, None));
builder.connect(file_in, 0, gain, 0);
```

### Stereo File

```rust
use bbx_dsp::{blocks::{FileInputBlock, GainBlock, PannerBlock}, graph::GraphBuilder};

let reader = WavFileReader::from_path("stereo.wav")?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);
let file_in = builder.add(FileInputBlock::new(Box::new(reader)));

// Connect both channels
let gain = builder.add(GainBlock::new(-6.0, None));
let pan = builder.add(PannerBlock::new(0.0));

// Left channel
builder.connect(file_in, 0, gain, 0);
// Right channel
builder.connect(file_in, 1, pan, 0);
```

### File to File Processing

```rust
use bbx_dsp::{blocks::{FileInputBlock, FileOutputBlock, OverdriveBlock}, graph::GraphBuilder};

let reader = WavFileReader::from_path("input.wav")?;
let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let file_in = builder.add(FileInputBlock::new(Box::new(reader)));
let effect = builder.add(OverdriveBlock::new(3.0, 1.0, 0.8, 44100.0));
let file_out = builder.add(FileOutputBlock::new(Box::new(writer)));

builder.connect(file_in, 0, effect, 0);
builder.connect(effect, 0, file_out, 0);

let mut graph = builder.build();

// Process all samples...
graph.finalize();
```

## Implementation Notes

- File is loaded into memory on creation
- Samples are read from internal buffer during process
- Looping behavior depends on implementation
- Returns zeros after file ends (no looping by default)
