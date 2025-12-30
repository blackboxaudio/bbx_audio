# SPSC Ring Buffer

A lock-free single-producer single-consumer ring buffer for inter-thread communication.

## Overview

The SPSC (Single-Producer Single-Consumer) ring buffer enables safe communication between two threads without locks. This is ideal for audio applications where:

- The audio thread produces samples
- The UI thread consumes visualization data
- Or vice versa

## API

### Creating a Buffer

```rust
use bbx_core::SpscRingBuffer;

// Create a buffer with capacity for 1024 samples
let (producer, consumer) = SpscRingBuffer::new::<f32>(1024);
```

The returned `producer` and `consumer` can be sent to different threads.

### Producer Operations

```rust
// Try to push a single item (non-blocking)
if producer.try_push(sample).is_ok() {
    // Sample was added
} else {
    // Buffer was full
}

// Push multiple items
let samples = [0.1, 0.2, 0.3];
let pushed = producer.try_push_slice(&samples);
// pushed = number of items actually added
```

### Consumer Operations

```rust
// Try to pop a single item (non-blocking)
if let Some(sample) = consumer.try_pop() {
    // Process sample
} else {
    // Buffer was empty
}

// Pop multiple items into a buffer
let mut output = [0.0f32; 256];
let popped = consumer.try_pop_slice(&mut output);
// popped = number of items actually read
```

### Checking Capacity

```rust
let available = consumer.len();     // Items ready to read
let space = producer.capacity() - producer.len();  // Space for writing
```

## Usage Patterns

### Audio Thread to UI Thread

```rust
use bbx_core::SpscRingBuffer;
use std::thread;

fn main() {
    let (producer, consumer) = SpscRingBuffer::new::<f32>(4096);

    // Audio thread
    let audio_handle = thread::spawn(move || {
        loop {
            let sample = generate_audio();
            let _ = producer.try_push(sample);
        }
    });

    // UI thread
    loop {
        while let Some(sample) = consumer.try_pop() {
            update_waveform_display(sample);
        }
    }
}
```

### MIDI Message Queue

```rust
use bbx_core::SpscRingBuffer;
use bbx_midi::MidiMessage;

let (midi_producer, midi_consumer) = SpscRingBuffer::new::<MidiMessage>(256);

// MIDI input callback
fn on_midi_message(msg: MidiMessage) {
    let _ = midi_producer.try_push(msg);
}

// Audio thread
fn process_audio() {
    while let Some(msg) = midi_consumer.try_pop() {
        handle_midi(msg);
    }
}
```

## Implementation Details

### Memory Ordering

The buffer uses atomic operations with appropriate memory ordering:

- `Relaxed` for capacity checks
- `Acquire`/`Release` for actual data transfer
- Ensures proper synchronization across threads

### Cache Efficiency

The producer and consumer indices are padded to avoid false sharing:

```rust
#[repr(align(64))]  // Cache line alignment
struct Producer<T> {
    write_index: AtomicUsize,
    // ...
}
```

### Wait-Free

Both push and pop operations are wait-free - they complete in bounded time regardless of what the other thread is doing.

## Limitations

- **Single producer only**: Multiple producers require external synchronization
- **Single consumer only**: Multiple consumers require external synchronization
- **Fixed capacity**: Size is set at creation, cannot grow
- **Power of two**: Capacity is rounded up to nearest power of two

For multiple producers or consumers, use a different data structure or add external synchronization.
