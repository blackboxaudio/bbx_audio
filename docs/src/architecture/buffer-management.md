# Buffer Management

How audio buffers are allocated and managed.

## Buffer Layout

Each block's outputs get contiguous buffer indices:

```
Block 0 (Oscillator): 1 output  -> Buffer 0
Block 1 (Panner):     2 outputs -> Buffers 1, 2
Block 2 (Output):     2 outputs -> Buffers 3, 4
```

## Pre-Allocation

Buffers are allocated when blocks are added:

```rust
pub fn add_block(&mut self, block: BlockType<S>) -> BlockId {
    let block_id = BlockId(self.blocks.len());

    // Record where this block's buffers start
    self.block_buffer_start.push(self.audio_buffers.len());
    self.blocks.push(block);

    // Allocate buffers for each output
    let output_count = self.blocks[block_id.0].output_count();
    for _ in 0..output_count {
        self.audio_buffers.push(AudioBuffer::new(self.buffer_size));
    }

    block_id
}
```

## Buffer Indexing

Fast O(1) lookup:

```rust
fn get_buffer_index(&self, block_id: BlockId, output_index: usize) -> usize {
    self.block_buffer_start[block_id.0] + output_index
}
```

## Connection Lookup

Input connections are pre-computed for O(1) access:

```rust
// Pre-computed during prepare()
self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
for conn in &self.connections {
    let buffer_idx = self.get_buffer_index(conn.from, conn.from_output);
    self.block_input_buffers[conn.to.0].push(buffer_idx);
}

// During processing - O(1) lookup
let input_indices = &self.block_input_buffers[block_id.0];
```

## Buffer Clearing

All buffers are zeroed at the start of each processing cycle:

```rust
for buffer in &mut self.audio_buffers {
    buffer.zeroize();
}
```

This allows multiple connections to the same input (signals are summed).

## Memory Efficiency

- Buffers are reused across processing cycles
- No allocations during `process_buffers()`
- Fixed memory footprint based on block count
