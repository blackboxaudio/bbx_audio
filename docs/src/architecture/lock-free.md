# Lock-Free Patterns

Concurrency patterns used in bbx_audio.

## Why Lock-Free?

Audio threads cannot wait for locks:

- Fixed time budget per buffer
- Priority inversion risks
- Unpredictable latency

## SPSC Ring Buffer

For single-producer single-consumer scenarios:

```rust
use bbx_core::SpscRingBuffer;

let (producer, consumer) = SpscRingBuffer::new::<f32>(1024);

// Producer thread
producer.try_push(sample);

// Consumer thread (audio)
if let Some(sample) = consumer.try_pop() {
    // Process
}
```

### Memory Ordering

```rust
// Simplified concept
impl SpscRingBuffer {
    fn try_push(&self, item: T) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);  // Sync with consumer

        if is_full(head, tail) {
            return false;
        }

        self.buffer[head] = item;
        self.head.store(next(head), Ordering::Release);  // Sync with consumer
        true
    }
}
```

## Atomic Parameters

For real-time parameter updates:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

struct Parameter {
    value: AtomicU32,
}

impl Parameter {
    fn set(&self, value: f32) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }
}
```

## When Locks Are OK

Not all code is audio-critical:

- **Setup/teardown**: Can use locks
- **UI thread**: Not time-critical
- **File I/O**: Background threads

Only the audio processing path must be lock-free.
