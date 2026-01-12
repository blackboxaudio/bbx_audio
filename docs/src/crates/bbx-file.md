# bbx_file

Audio file I/O for the bbx_audio workspace.

## Overview

bbx_file provides:

- WAV file reading via `wavers`
- WAV file writing via `hound`
- Integration with bbx_dsp blocks

## Installation

```toml
[dependencies]
bbx_file = "0.1"
bbx_dsp = "0.1"
```

## Supported Formats

| Format | Read | Write |
|--------|------|-------|
| WAV    | Yes  | Yes   |

## Features

| Feature | Description |
|---------|-------------|
| [WAV Reader](file/wav-reader.md) | Load WAV files |
| [WAV Writer](file/wav-writer.md) | Create WAV files |

## Quick Example

### Reading

```rust
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("audio.wav")?;

println!("Sample rate: {}", reader.sample_rate());
println!("Channels: {}", reader.num_channels());
println!("Duration: {:.2}s", reader.duration_seconds());

let left_channel = reader.read_channel(0);
```

### Writing

```rust
use bbx_file::writers::wav::WavFileWriter;

let mut writer = WavFileWriter::<f32>::new("output.wav", 44100.0, 2)?;

writer.write_channel(0, &left_samples)?;
writer.write_channel(1, &right_samples)?;

writer.finalize()?;
```

### With bbx_dsp

```rust
use bbx_dsp::{block::BlockType, blocks::GainBlock, graph::GraphBuilder};
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("input.wav")?;
let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let file_in = builder.add_file_input(Box::new(reader));
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

builder.connect(file_in, 0, gain, 0);

let graph = builder.build();
```
