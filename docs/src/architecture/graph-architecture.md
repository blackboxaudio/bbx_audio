# DSP Graph Architecture

The core design of bbx_audio's DSP processing system.

## Overview

bbx_audio uses a directed acyclic graph (DAG) architecture where:

1. **Blocks** are processing nodes
2. **Connections** define signal flow
3. **Topological sorting** determines execution order
4. **Pre-allocated buffers** enable real-time processing

## Key Components

### Graph

The `Graph` struct manages:

```rust
pub struct Graph<S: Sample> {
    blocks: Vec<BlockType<S>>,           // All DSP blocks
    connections: Vec<Connection>,         // Block connections
    execution_order: Vec<BlockId>,        // Sorted processing order
    output_block: Option<BlockId>,        // Final output
    audio_buffers: Vec<AudioBuffer<S>>,   // Pre-allocated buffers
    modulation_values: Vec<S>,            // Per-block modulation
}
```

### GraphBuilder

Fluent API for construction:

```rust
use bbx_dsp::{
    block::BlockType,
    blocks::GainBlock,
    graph::GraphBuilder,
    waveform::Waveform,
};

let mut builder = GraphBuilder::<f32>::new(44100.0, 512, 2);

let osc = builder.add_oscillator(440.0, Waveform::Sine, None);
let gain = builder.add_block(BlockType::Gain(GainBlock::new(-6.0, None)));

builder.connect(osc, 0, gain, 0);
let graph = builder.build();
```

### Connection

Describes signal routing:

```rust
pub struct Connection {
    pub from: BlockId,        // Source block
    pub from_output: usize,   // Output port
    pub to: BlockId,          // Destination block
    pub to_input: usize,      // Input port
}
```

## Processing Pipeline

1. **Clear buffers** - Zero all audio buffers
2. **Execute blocks** - Process in topological order
3. **Collect modulation** - Gather modulator outputs
4. **Copy output** - Transfer to user buffers

```rust
pub fn process_buffers(&mut self, output_buffers: &mut [&mut [S]]) {
    // Clear all buffers
    for buffer in &mut self.audio_buffers {
        buffer.zeroize();
    }

    // Process blocks in order
    for block_id in &self.execution_order {
        self.process_block(*block_id);
        self.collect_modulation_values(*block_id);
    }

    // Copy to output
    self.copy_to_output_buffer(output_buffers);
}
```

## Design Decisions

### Pre-allocation

All buffers are allocated during `prepare_for_playback()`:

- No allocations during processing
- Fixed buffer sizes
- Predictable memory usage

### Stack-Based I/O

Input/output slices use stack allocation:

```rust
const MAX_BLOCK_INPUTS: usize = 8;
const MAX_BLOCK_OUTPUTS: usize = 8;

let mut input_slices: StackVec<&[S], MAX_BLOCK_INPUTS> = StackVec::new();
```

### Buffer Indexing

Each block has a contiguous range of buffers:

```rust
fn get_buffer_index(&self, block_id: BlockId, output_index: usize) -> usize {
    self.block_buffer_start[block_id.0] + output_index
}
```

## Related Topics

- [Topological Sorting](topological-sort.md) - Execution order algorithm
- [Buffer Management](buffer-management.md) - Buffer allocation strategy
- [Real-Time Safety](realtime-safety.md) - Audio thread constraints
