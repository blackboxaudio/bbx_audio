# Zero-Allocation Processing

How bbx_audio achieves zero allocations during audio processing.

## Strategy

All memory allocated upfront:

1. **Blocks added** → buffers allocated
2. **Graph prepared** → connection lookups built
3. **Processing** → only use pre-allocated memory

## Pre-Allocated Resources

### Audio Buffers

```rust
// Allocated when block is added
for _ in 0..output_count {
    self.audio_buffers.push(AudioBuffer::new(self.buffer_size));
}

// During processing - just clear
buffer.zeroize();
```

### Modulation Values

```rust
// Allocated during prepare
self.modulation_values.resize(self.blocks.len(), S::ZERO);

// During processing - just write
self.modulation_values[block_id.0] = value;
```

### Connection Lookups

```rust
// Computed during prepare
self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
for conn in &self.connections {
    self.block_input_buffers[conn.to.0].push(buffer_idx);
}

// During processing - O(1) read
let inputs = &self.block_input_buffers[block_id.0];
```

## Stack Allocation

Temporary collections use stack memory:

```rust
const MAX_BLOCK_INPUTS: usize = 8;

// No heap allocation
let mut input_slices: StackVec<&[S], MAX_BLOCK_INPUTS> = StackVec::new();
```

## Verification

Check with a global allocator hook:

```rust
#[cfg(test)]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn test_no_allocations_during_process() {
    // Setup...
    let before = dhat::total_allocations();
    graph.process_buffers(&mut outputs);
    let after = dhat::total_allocations();
    assert_eq!(before, after, "Allocations during process!");
}
```
