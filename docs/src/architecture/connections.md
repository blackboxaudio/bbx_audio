# Connection System

How blocks are connected in the DSP graph.

## Connection Structure

```rust
pub struct Connection {
    pub from: BlockId,        // Source block
    pub from_output: usize,   // Which output port
    pub to: BlockId,          // Destination block
    pub to_input: usize,      // Which input port
}
```

## Port Numbering

Ports are zero-indexed:

```
OscillatorBlock:
  Outputs: [0] (mono audio)

PannerBlock:
  Inputs:  [0] (mono in)
  Outputs: [0] (left), [1] (right)
```

## Making Connections

Via GraphBuilder:

```rust
builder.connect(
    oscillator_id,  // from
    0,              // from_output
    panner_id,      // to
    0,              // to_input
);
```

## Connection Rules

### One-to-Many

An output can connect to multiple inputs:

```rust
builder.connect(osc, 0, gain1, 0);
builder.connect(osc, 0, gain2, 0);  // Same output, different targets
```

### Many-to-One

Multiple outputs can connect to the same input (summed):

```rust
builder.connect(osc1, 0, mixer, 0);
builder.connect(osc2, 0, mixer, 0);  // Both summed into mixer input
```

### No Cycles

Connections must form a DAG (directed acyclic graph):

```
Valid:    A -> B -> C
Invalid:  A -> B -> A (cycle!)
```

## Signal Summing

When multiple connections go to the same input:

1. Buffers start zeroed
2. Each source adds to the buffer
3. Result is the sum of all inputs

```rust
// Conceptually:
output_buffer[sample] += input_buffer[sample];
```

## Validation

Connection validity is checked during `build()`:

- Source and destination blocks must exist
- Port indices must be valid
- Graph must be acyclic
