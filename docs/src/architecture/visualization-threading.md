# Visualization Threading Model

This document describes the thread-safe communication patterns between audio and visualization threads in bbx_draw.

## The Challenge

Audio and visualization run on different threads with different constraints:

| Thread | Priority | Deadline | Blocking |
|--------|----------|----------|----------|
| Audio | Real-time | Fixed (e.g., 5.8ms at 44.1kHz/256 samples) | Never allowed |
| Visualization | Best-effort | ~16ms (60 FPS) | Acceptable |

The audio thread cannot wait for the visualization thread. Any blocking would cause audio dropouts.

## The Solution: SPSC Ring Buffer

bbx_draw uses single-producer single-consumer (SPSC) ring buffers from `bbx_core`:

```
┌─────────────────────┐         SPSC Ring Buffer        ┌─────────────────────┐
│    Audio Thread     │ ────────────────────────────▶   │  nannou Thread      │
│  (rodio callback)   │     AudioFrame packets          │  (model/update/view)│
│                     │                                 │                     │
│  try_send()         │                                 │  try_pop()          │
│  (non-blocking)     │                                 │  (consume all)      │
└─────────────────────┘                                 └─────────────────────┘
```

The ring buffer uses atomic operations for lock-free synchronization. See [Lock-Free Patterns](lock-free.md) for implementation details.

## AudioBridgeProducer

The producer runs in the audio thread:

```rust
impl AudioBridgeProducer {
    pub fn try_send(&mut self, frame: Frame) -> bool {
        self.producer.try_push(frame).is_ok()
    }
}
```

Key properties:
- **Non-blocking**: `try_send()` returns immediately
- **No allocation**: Frame is moved, not copied
- **Graceful overflow**: Returns `false` if buffer full

### Audio Thread Rules

```rust
// In audio callback - runs at audio rate (e.g., 44100/512 = ~86x per second)
fn audio_callback(producer: &mut AudioBridgeProducer, samples: &[f32]) {
    let frame = Frame::new(samples, 44100, 2);
    producer.try_send(frame);  // Don't check result
}
```

Guidelines:
- Never allocate memory
- Always use `try_send()`
- Don't check `is_full()` before sending
- Dropped frames are acceptable

## AudioBridgeConsumer

The consumer runs in the visualization thread:

```rust
impl AudioBridgeConsumer {
    pub fn try_pop(&mut self) -> Option<Frame> {
        self.consumer.try_pop()
    }
}
```

Key properties:
- **Non-blocking**: Returns `None` if empty
- **Batch processing**: Call repeatedly to drain
- **No contention**: Single consumer thread

### Visualization Thread Pattern

```rust
// In visualizer update() - runs at frame rate (e.g., 60 FPS)
fn update(&mut self) {
    while let Some(frame) = self.consumer.try_pop() {
        self.process_frame(&frame);
    }
}
```

Process all available frames each visual frame to prevent buffer buildup.

## Rate Mismatch

Audio generates data faster than visualization consumes it:

| Source | Rate | Data Per Visual Frame |
|--------|------|----------------------|
| Audio (44.1kHz, 512 samples) | ~86 frames/sec | ~1.4 frames |
| Visualization | 60 frames/sec | 1 frame |

The visualization thread receives ~1-2 audio frames per visual frame on average. Bursts may be larger.

## Capacity Tuning

Bridge capacity affects latency and reliability:

### Audio Bridge

```rust
let (producer, consumer) = audio_bridge(capacity);
```

| Capacity | Latency | Reliability |
|----------|---------|-------------|
| 4 | ~5ms | May drop during load |
| 8 | ~10ms | Balanced |
| 16 | ~20ms | Very reliable |

Recommended: 8-16 frames for typical use.

### MIDI Bridge

```rust
let (producer, consumer) = midi_bridge(capacity);
```

MIDI is bursty (note-on/off events), not continuous:

| Capacity | Notes |
|----------|-------|
| 64 | Light use |
| 256 | Heavy chords, fast playing |
| 512 | Recording, dense sequences |

Recommended: 256 messages for general use.

## Frame Dropping

Frame drops are acceptable for visualization:

```
Audio: [F1][F2][F3][F4][F5][F6]...
       ↓   ↓   ↓   ↓   ↓   ↓
       ✓   ✓   X   ✓   X   ✓    (X = dropped)
Visual: [F1,F2] [F4] [F6]        (batched per visual frame)
```

The human eye won't notice missing frames when 60 visual frames per second are rendered. Audio dropouts are far more noticeable.

## The Frame Type

`Frame` (aliased as `AudioFrame` in bbx_draw) uses stack allocation:

```rust
pub struct Frame {
    pub samples: StackVec<f32, MAX_FRAME_SAMPLES>,
    pub sample_rate: u32,
    pub channels: usize,
}
```

`StackVec` avoids heap allocation, making frame creation real-time safe.

## MIDI vs Audio Bridges

The bridges have different drain patterns:

### Audio: Pop Loop

```rust
while let Some(frame) = consumer.try_pop() {
    // Process each frame
}
```

Processes frames individually, suitable for buffer accumulation.

### MIDI: Drain All

```rust
let messages = consumer.drain();  // Returns Vec<MidiMessage>
for msg in messages {
    // Process each message
}
```

Collects all messages at once. Note: `drain()` allocates a `Vec`, acceptable for the visualization thread.

## Best Practices

### Audio Thread

- Never allocate (use `Frame::new()` which uses stack)
- Always call `try_send()`, never block
- Don't log, print, or acquire locks
- Keep processing deterministic

### Visualization Thread

- Drain all available data each frame
- Don't hold references to frame data across updates
- Use smoothing/interpolation for visual continuity
- Batch expensive operations

### General

- Size buffers for worst-case burst, not average
- Profile actual drop rates under load
- Consider separate bridges for different visualizers
- Test with intentional load to verify behavior
