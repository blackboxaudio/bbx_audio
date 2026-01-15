# Backend Trait

The `Backend` trait defines how audio samples are sent to the output device.

## Backend Trait

```rust
pub trait Backend: Send + 'static {
    fn play(
        self: Box<Self>,
        signal: Box<dyn Iterator<Item = f32> + Send>,
        sample_rate: u32,
        num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()>;
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `signal` | `Box<dyn Iterator<Item = f32> + Send>` | Interleaved f32 samples |
| `sample_rate` | `u32` | Sample rate in Hz |
| `num_channels` | `u16` | Number of output channels |
| `stop_flag` | `Arc<AtomicBool>` | Shared flag to signal stop |

### Implementation Notes

- The `self: Box<Self>` signature allows backends to move ownership into a background thread
- Backends should check `stop_flag` periodically and terminate when it's `true`
- All samples are converted to `f32` before reaching the backend

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
use bbx_player::{Backend, Result};
use std::sync::{Arc, atomic::AtomicBool};

struct MyBackend {
    // Custom state
}

impl Backend for MyBackend {
    fn play(
        self: Box<Self>,
        signal: Box<dyn Iterator<Item = f32> + Send>,
        sample_rate: u32,
        num_channels: u16,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<()> {
        // Spawn background thread
        std::thread::spawn(move || {
            // Process samples from signal iterator
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
