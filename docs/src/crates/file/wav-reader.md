# WAV Reader

Load WAV files for processing with bbx_dsp.

## Creating a Reader

```rust
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("audio.wav")?;
```

## File Information

```rust
use bbx_file::readers::wav::WavFileReader;
use bbx_dsp::reader::Reader;

let reader = WavFileReader::<f32>::from_path("audio.wav")?;

// Sample rate in Hz
let rate = reader.sample_rate();

// Number of channels
let channels = reader.num_channels();

// Total samples per channel
let samples = reader.num_samples();

// Duration
let duration = reader.duration_seconds();
```

## Reading Audio Data

### Full Channel

```rust
let left = reader.read_channel(0);
let right = reader.read_channel(1);
```

### Partial Read

```rust
// Read specific range
let data = reader.read_range(0, 1000..2000);
```

## Reader Trait

`WavFileReader` implements the `Reader` trait from bbx_dsp:

```rust
pub trait Reader<S: Sample>: Send {
    fn sample_rate(&self) -> f64;
    fn num_channels(&self) -> usize;
    fn num_samples(&self) -> usize;
    fn duration_seconds(&self) -> f64;
    fn read_channel(&self, channel: usize) -> Vec<S>;
}
```

## Usage with FileInputBlock

```rust
use bbx_dsp::{blocks::FileInputBlock, graph::GraphBuilder};
use bbx_file::readers::wav::WavFileReader;

let reader = WavFileReader::<f32>::from_path("input.wav")?;

let mut builder = GraphBuilder::<f32>::new(
    reader.sample_rate(),
    512,
    reader.num_channels(),
);

let file_input = builder.add(FileInputBlock::new(Box::new(reader)));
```

## Supported Formats

- PCM 8-bit unsigned
- PCM 16-bit signed
- PCM 24-bit signed
- PCM 32-bit signed
- IEEE Float 32-bit
- IEEE Float 64-bit

## Error Handling

```rust
use bbx_file::readers::wav::WavFileReader;

match WavFileReader::<f32>::from_path("audio.wav") {
    Ok(reader) => {
        // Use reader
    }
    Err(e) => {
        eprintln!("Failed to load: {}", e);
    }
}
```

## Performance Notes

- Files are loaded entirely into memory
- For large files, consider streaming approaches
- Sample type conversion happens on load
