# I/O Blocks

I/O blocks handle audio input and output for graphs.

## Available I/O Blocks

| Block | Description |
|-------|-------------|
| [FileInputBlock](io/file-input.md) | Read from audio files |
| [FileOutputBlock](io/file-output.md) | Write to audio files |
| [OutputBlock](io/output.md) | Graph audio output |

## Block Roles

### Source (FileInputBlock)

Provides audio from external files:

```rust
let reader = WavFileReader::from_path("input.wav")?;
let input = builder.add_file_input(Box::new(reader));
```

### Sink (FileOutputBlock)

Writes audio to external files:

```rust
let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;
let output = builder.add_file_output(Box::new(writer));
```

### Terminal (OutputBlock)

Collects audio for real-time output or further processing:

```rust
// Automatically added by GraphBuilder when building
let graph = builder.build();
// Output is collected via process_buffers()
```

## Usage Patterns

### File Processing

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
};
use bbx_file::{readers::wav::WavFileReader, writers::wav::WavFileWriter};

// Set up I/O
let reader = WavFileReader::from_path("input.wav")?;
let writer = WavFileWriter::new("output.wav", 44100.0, 2)?;

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

// Create graph: Input -> Effect -> Output
let file_in = builder.add_file_input(Box::new(reader));
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));
let file_out = builder.add_file_output(Box::new(writer));

builder.connect(file_in, 0, gain, 0);
builder.connect(gain, 0, file_out, 0);

let mut graph = builder.build();

// Process entire file
loop {
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];
    let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
    graph.process_buffers(&mut outputs);

    // Check for end of file...
}

// Finalize output file
graph.finalize();
```

### Real-Time Processing

For real-time (plugin, live), use OutputBlock implicitly:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
// OutputBlock added automatically

let mut graph = builder.build();

// Audio callback
fn process(graph: &mut Graph, output: &mut AudioBuffer) {
    graph.process_buffers(&mut output.channels());
}
```
