# Audio Bridge

Lock-free communication between audio and visualization threads.

## Overview

The audio bridge provides thread-safe, real-time-safe transfer of audio data using `bbx_core::SpscRingBuffer`. The producer runs in the audio thread, while the consumer runs in the visualization thread.

## Creating a Bridge

```rust
use bbx_draw::{audio_bridge, AudioBridgeProducer, AudioBridgeConsumer};

// Create producer/consumer pair with capacity of 16 frames
let (producer, consumer) = audio_bridge(16);
```

The `capacity` parameter determines how many audio frames can be buffered. Typical values are 4-16 frames.

## AudioBridgeProducer

Used in the audio thread to send frames.

### Methods

| Method | Description |
|--------|-------------|
| `try_send(&mut self, frame: Frame) -> bool` | Send a frame (non-blocking) |
| `is_full(&self) -> bool` | Check if buffer is full |

### Usage

```rust
// In audio callback
let frame = Frame::new(&samples, 44100, 2);
producer.try_send(frame);  // Don't block if full
```

`try_send()` returns `false` if the buffer is full. Dropping frames is acceptable for visualization.

## AudioBridgeConsumer

Used in the visualization thread to receive frames.

### Methods

| Method | Description |
|--------|-------------|
| `try_pop(&mut self) -> Option<Frame>` | Pop one frame (non-blocking) |
| `is_empty(&self) -> bool` | Check if buffer is empty |
| `len(&self) -> usize` | Number of available frames |

### Usage

```rust
// In visualizer update()
while let Some(frame) = consumer.try_pop() {
    // Process frame
    for sample in frame.samples.iter() {
        // ...
    }
}
```

## MIDI Bridge

For MIDI visualization, use the MIDI bridge:

```rust
use bbx_draw::{midi_bridge, MidiBridgeProducer, MidiBridgeConsumer};

let (producer, consumer) = midi_bridge(256);
```

### MidiBridgeProducer

| Method | Description |
|--------|-------------|
| `try_send(&mut self, msg: MidiMessage) -> bool` | Send a MIDI message |
| `is_full(&self) -> bool` | Check if buffer is full |

### MidiBridgeConsumer

| Method | Description |
|--------|-------------|
| `drain(&mut self) -> Vec<MidiMessage>` | Get all available messages |
| `is_empty(&self) -> bool` | Check if buffer is empty |

## Capacity Guidelines

| Bridge Type | Recommended Capacity | Notes |
|-------------|---------------------|-------|
| Audio | 4-16 frames | Higher = more latency |
| MIDI | 64-256 messages | MIDI is sparse, bursty |

### Tradeoffs

**Too small**: Frame drops cause visual stuttering

**Too large**: Increased latency between audio and visual

## Complete Example

```rust
use bbx_draw::{audio_bridge, AudioFrame, WaveformVisualizer, Visualizer};
use std::thread;

fn main() {
    // Create bridge
    let (mut producer, consumer) = audio_bridge(16);

    // Spawn audio thread
    thread::spawn(move || {
        loop {
            let samples = generate_audio();
            let frame = AudioFrame::new(&samples, 44100, 2);
            producer.try_send(frame);
        }
    });

    // Create visualizer with consumer
    let mut visualizer = WaveformVisualizer::new(consumer);

    // In visualization loop
    visualizer.update();  // Consumes frames from bridge
    // visualizer.draw(...);
}
```

## Frame Type

`AudioFrame` (alias for `bbx_dsp::Frame`) contains:

| Field | Type | Description |
|-------|------|-------------|
| `samples` | `StackVec<S, MAX_FRAME_SAMPLES>` | Audio samples |
| `sample_rate` | `u32` | Sample rate in Hz |
| `channels` | `usize` | Number of channels |

The `StackVec` uses stack allocation for real-time safety.
