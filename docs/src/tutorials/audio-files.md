# Working with Audio Files

This tutorial covers reading and writing audio files with bbx_audio.

## Prerequisites

Add bbx_file to your project:

```toml
[dependencies]
bbx_dsp = "0.1"
bbx_file = "0.1"
```

## Supported Formats

| Format | Read | Write |
|--------|------|-------|
| WAV | Yes | Yes |

## Reading WAV Files

### Creating a File Reader

```rust
use bbx_file::readers::wav::WavFileReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = WavFileReader::from_path("audio/sample.wav")?;

    // Get file information
    println!("Sample rate: {}", reader.sample_rate());
    println!("Channels: {}", reader.num_channels());
    println!("Duration: {} seconds", reader.duration_seconds());

    Ok(())
}
```

### Using FileInputBlock

Add a file input to your DSP graph:

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::readers::wav::WavFileReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Create the reader
    let reader = WavFileReader::from_path("audio/sample.wav")?;

    // Add to graph
    let file_input = builder.add_file_input(Box::new(reader));

    // Connect to effects
    let gain = builder.add_gain(-6.0);
    builder.connect(file_input, 0, gain, 0);

    let graph = builder.build();
    Ok(())
}
```

## Writing WAV Files

### Creating a File Writer

```rust
use bbx_file::writers::wav::WavFileWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let writer = WavFileWriter::new(
        "output.wav",
        44100,  // Sample rate
        2,      // Channels
        16,     // Bits per sample
    )?;

    Ok(())
}
```

### Using FileOutputBlock

Add a file output to your DSP graph:

```rust
use bbx_dsp::{graph::GraphBuilder, waveform::Waveform};
use bbx_file::writers::wav::WavFileWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

    // Add oscillator
    let osc = builder.add_oscillator(440.0, Waveform::Sine, None);

    // Create writer
    let writer = WavFileWriter::new("output.wav", 44100, 2, 16)?;

    // Add file output block
    let file_output = builder.add_file_output(Box::new(writer));

    // Connect oscillator to file output
    builder.connect(osc, 0, file_output, 0);

    let mut graph = builder.build();

    // Process and write audio
    let mut left = vec![0.0f32; 512];
    let mut right = vec![0.0f32; 512];

    // Write 5 seconds of audio
    let num_buffers = (5.0 * 44100.0 / 512.0) as usize;
    for _ in 0..num_buffers {
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        graph.process_buffers(&mut outputs);
    }

    // Finalize the file
    graph.finalize();

    Ok(())
}
```

## Processing Audio Files

Combine file input and output for offline processing:

```rust
use bbx_dsp::graph::GraphBuilder;
use bbx_file::{
    readers::wav::WavFileReader,
    writers::wav::WavFileWriter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open input file
    let reader = WavFileReader::from_path("input.wav")?;
    let sample_rate = reader.sample_rate();
    let num_channels = reader.num_channels();

    // Create graph
    let mut builder = GraphBuilder::<f32>::new(sample_rate, 512, num_channels);

    // File input
    let file_in = builder.add_file_input(Box::new(reader));

    // Process: add some effects
    let gain = builder.add_gain(-3.0);
    let pan = builder.add_panner(0.25);

    builder.connect(file_in, 0, gain, 0);
    builder.connect(gain, 0, pan, 0);

    // File output
    let writer = WavFileWriter::new("output.wav", sample_rate as u32, num_channels, 16)?;
    let file_out = builder.add_file_output(Box::new(writer));

    builder.connect(pan, 0, file_out, 0);

    let mut graph = builder.build();

    // Process entire file
    let mut outputs = vec![vec![0.0f32; 512]; num_channels];

    loop {
        let mut output_refs: Vec<&mut [f32]> = outputs.iter_mut()
            .map(|v| v.as_mut_slice())
            .collect();

        graph.process_buffers(&mut output_refs);

        // Check if file input is exhausted
        // (implementation depends on your needs)
        break;  // Placeholder
    }

    graph.finalize();
    Ok(())
}
```

## Error Handling

Handle common file errors:

```rust
use bbx_file::readers::wav::WavFileReader;

fn main() {
    match WavFileReader::from_path("nonexistent.wav") {
        Ok(reader) => {
            println!("Loaded: {} channels, {} Hz",
                reader.num_channels(),
                reader.sample_rate());
        }
        Err(e) => {
            eprintln!("Failed to load audio file: {}", e);
        }
    }
}
```

## Performance Tips

1. **Buffer size**: Use larger buffers for file processing (2048+ samples)
2. **Non-blocking I/O**: `FileOutputBlock` uses non-blocking I/O internally
3. **Memory**: Large files are streamed, not loaded entirely into memory
4. **Finalization**: Always call `finalize()` to flush buffers and close files

## Next Steps

- [MIDI Integration](midi.md) - Control playback with MIDI
- [JUCE Integration](../juce/overview.md) - Use in a plugin
