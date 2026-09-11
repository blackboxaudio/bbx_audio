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
    self.audio_buffers.push(SampleBuffer::new(self.buffer_size));
}

// During processing - just clear
buffer.zeroize();
```

### Modulation Values

```rust
// Allocated during prepare: one slot per modulation output, plus an offset table
self.modulation_values.resize(total_modulation_outputs, S::ZERO);

// During processing - just write
self.modulation_values[offsets[block_id.0] + output] = value;
```

### Connection Lookups

```rust
// Computed during prepare: one slot per input port, indexed by port number
self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
for connection in &self.connections {
    let ports = &mut self.block_input_buffers[connection.to.0];
    ports.resize(ports.len().max(connection.to_input + 1), None);
    ports[connection.to_input] = Some(buffer_index);
}

// During processing - O(1) read; None becomes an empty slice
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
