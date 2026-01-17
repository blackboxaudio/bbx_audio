# Stack Allocation Strategy

How bbx_audio avoids heap allocations during processing.

## The Problem

Audio processing must avoid heap allocation because:

- `malloc()`/`free()` can take unbounded time
- System allocator may lock
- Heap fragmentation over time

## StackVec Solution

`StackVec<T, N>` provides vector-like behavior with stack storage:

```rust
const MAX_BLOCK_INPUTS: usize = 8;
let mut input_slices: StackVec<&[S], MAX_BLOCK_INPUTS> = StackVec::new();

for &index in input_indices {
    input_slices.push_unchecked(/* slice */);
}
```

## Fixed Capacity Limits

Blocks are limited to reasonable I/O counts:

```rust
const MAX_BLOCK_INPUTS: usize = 8;
const MAX_BLOCK_OUTPUTS: usize = 8;
```

These limits are validated during `build()`:

```rust
assert!(
    connected_inputs <= MAX_BLOCK_INPUTS,
    "Block {idx} has too many inputs"
);
```

## Pre-Allocation Pattern

Storage allocated once, reused forever:

```rust
// In Graph struct
audio_buffers: Vec<AudioBuffer<S>>,
modulation_values: Vec<S>,

// In prepare()
self.modulation_values.resize(self.blocks.len(), S::ZERO);

// In process_buffers() - just clear, no resize
for buffer in &mut self.audio_buffers {
    buffer.zeroize();
}
```

## Trade-offs

### Advantages

- Zero allocations during processing
- Predictable memory layout
- No allocator contention

### Disadvantages

- Fixed maximum I/O counts
- Higher upfront memory usage
- Must know limits at compile time
