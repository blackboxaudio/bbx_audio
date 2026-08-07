# Cache Efficiency

Optimizing for CPU cache performance.

## Cache Hierarchy

| Level | Size | Latency |
|-------|------|---------|
| L1 | 32-64 KB | ~4 cycles |
| L2 | 256-512 KB | ~12 cycles |
| L3 | 4-32 MB | ~40 cycles |
| RAM | GBs | ~200+ cycles |

## Strategies

### Contiguous Storage

Keep related data together:

```rust
// Good: Single contiguous allocation
audio_buffers: Vec<SampleBuffer<S>>

// Each buffer is contiguous
samples: [S; BUFFER_SIZE]
```

### Sequential Access

Process in order:

```rust
// Good: Sequential iteration
for sample in buffer.iter_mut() {
    *sample = process(*sample);
}

// Avoid: Random access
for i in random_order {
    buffer[i] = process(buffer[i]);
}
```

### Hot/Cold Separation

Separate frequently from rarely used data:

```rust
struct Block<S> {
    // Hot path (processing)
    state: S,
    coefficient: S,

    // Cold path (setup)
    name: String,  // Rarely accessed
}
```

### Avoid Pointer Chasing

Minimize indirection:

```rust
// Less ideal: Vec of trait objects
blocks: Vec<Box<dyn Block>>

// Better: Enum of concrete types
blocks: Vec<BlockType<S>>
```

## Buffer Layout

Interleaved vs non-interleaved:

```rust
// Non-interleaved (better for processing)
left:  [L0, L1, L2, L3, ...]
right: [R0, R1, R2, R3, ...]

// Interleaved (worse for SIMD)
data: [L0, R0, L1, R1, L2, R2, ...]
```

bbx_audio uses non-interleaved buffers.
