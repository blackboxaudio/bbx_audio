# Backend Trait

The `Backend` trait defines how audio samples are sent to the output device.

## Backend Trait

```rust
pub trait Backend<S: Sample>: Send + 'static {
    fn play(
        self: Box<Self>,
        source: Box<dyn Source<S>>,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()>;
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source` | `Box<dyn Source<S>>` | Audio source providing interleaved samples and metadata |
| `stop_flag` | `Arc<AtomicBool>` | Shared flag to signal stop |

### The Source Trait

The `Source<S>` trait combines an iterator of samples with audio metadata:

```rust
pub trait Source<S: Sample>: Iterator<Item = S> + Send + 'static {
    fn channels(&self) -> u16;
    fn sample_rate(&self) -> u32;
}
```

Backends obtain channel count and sample rate from the source rather than as separate parameters.

### Implementation Notes

- The trait is generic over `S: Sample`, allowing backends to work with any sample type
- The `self: Box<Self>` signature allows backends to move ownership into a background thread
- Backends should check `stop_flag` periodically and terminate when it's `true`

## Built-in Backends

### RodioBackend

High-level backend using [rodio](https://crates.io/crates/rodio). This is the default backend.

```rust
use bbx_player::backends::RodioBackend;

let backend = RodioBackend::try_default()?;
let player = Player::with_backend(graph, backend);
```

**Characteristics:**

- Automatic device selection
- Simple setup
- Uses rodio's `Source` trait internally

**When to use:**

- Quick prototyping
- When device selection doesn't matter
- Most common use cases

### CpalBackend

Low-level backend using [cpal](https://crates.io/crates/cpal) directly.

```rust
use bbx_player::backends::CpalBackend;

let backend = CpalBackend::try_default()?;
let player = Player::with_backend(graph, backend);
```

**Characteristics:**

- Direct access to cpal APIs
- Uses a lock-free ring buffer for audio thread communication
- Separate producer thread fills the buffer

**When to use:**

- Need specific device selection
- Building on top of cpal APIs
- Need more control over audio thread behavior

## Comparison

| Aspect | RodioBackend | CpalBackend |
|--------|--------------|-------------|
| Setup complexity | Low | Medium |
| Device control | Automatic | Direct access |
| Dependencies | rodio | cpal, bbx_core |
| Buffer management | Handled by rodio | Lock-free ring buffer |
| Thread model | Single playback thread | Producer + callback threads |

## Custom Backends

You can implement the `Backend` trait for custom audio output:

```rust
use bbx_dsp::sample::Sample;
use bbx_player::{Backend, Result, Source};
use std::sync::{Arc, atomic::AtomicBool};

struct MyBackend {
    // Custom state
}

impl<S: Sample> Backend<S> for MyBackend {
    fn play(
        self: Box<Self>,
        source: Box<dyn Source<S>>,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()> {
        let channels = source.channels();
        let sample_rate = source.sample_rate();

        // Spawn background thread
        std::thread::spawn(move || {
            // Process samples from source iterator
            // Check stop_flag periodically
            // Send to your audio output
        });

        Ok(())
    }
}
```

### Requirements

1. **`Send + 'static`**: Backend must be sendable to other threads
2. **Non-blocking**: `play()` should return immediately after spawning the playback thread
3. **Stop flag**: Must respect the `stop_flag` to allow clean shutdown
4. **Thread ownership**: The `Box<Self>` signature means the backend is consumed
