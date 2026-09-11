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
        self.audio_buffers.push(SampleBuffer::new(self.buffer_size));
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
// Pre-computed during prepare(), indexed by the destination *port* so a block
// sees each input at the port it declared, whatever order it was connected in
self.block_input_buffers = vec![Vec::new(); self.blocks.len()];
for connection in &self.connections {
    let buffer_index = self.get_buffer_index(connection.from, connection.from_output);
    let ports = &mut self.block_input_buffers[connection.to.0];
    if ports.len() <= connection.to_input {
        ports.resize(connection.to_input + 1, None);
    }
    assert!(ports[connection.to_input].is_none(), "port connected twice");
    ports[connection.to_input] = Some(buffer_index);
}

// During processing - O(1) lookup; None becomes an empty slice
let input_indices = &self.block_input_buffers[block_id.0];
```

An unconnected port below the highest connected one reads as an empty slice, so each
block's own "missing input" default applies (`VcaBlock` treats a missing control as
unity, a missing audio input as silence). Connecting two sources to one port is rejected
at build time; sum them with a `MixerBlock`.

`prepare()` also re-creates every buffer whose length differs from the new buffer size,
so a host that changes its block size after the graph was built is handled.

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
