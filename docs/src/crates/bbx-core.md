# bbx_core

Foundational utilities and data structures for the bbx_audio workspace.

## Overview

bbx_core provides low-level utilities designed for real-time audio applications:

- Denormal handling to prevent CPU slowdowns
- Lock-free data structures for inter-thread communication
- Stack-allocated containers to avoid heap allocations
- Fast random number generation

## Installation

```toml
[dependencies]
bbx_core = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| [Sample](core/sample.md) | Generic sample type trait with SIMD support |
| [SIMD](../architecture/simd.md) | Vectorized DSP operations (feature-gated) |
| [Denormal Handling](core/denormal.md) | Flush denormal floats to zero |
| [SPSC Ring Buffer](core/spsc.md) | Lock-free producer-consumer queue |
| [Stack Vector](core/stack-vec.md) | Fixed-capacity heap-free vector |
| [Random](core/random.md) | Fast XorShift RNG |
| [Error Types](core/error.md) | Unified error handling |

## Quick Example

```rust
use bbx_core::{StackVec, flush_denormal_f32};

// Stack-allocated buffer for audio processing
let mut samples: StackVec<f32, 256> = StackVec::new();

// Process samples, flushing denormals
for sample in &mut samples {
    *sample = flush_denormal_f32(*sample);
}
```

## Design Principles

### No Heap Allocations

All data structures are designed to work without heap allocation during audio processing. Buffers are pre-allocated and reused.

### Lock-Free

The SPSC ring buffer uses atomic operations instead of mutexes, ensuring consistent performance.

### Minimal Dependencies

bbx_core has minimal external dependencies, keeping compile times low and reducing potential issues.
