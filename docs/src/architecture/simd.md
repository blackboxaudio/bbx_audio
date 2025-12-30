# SIMD Opportunities

Single Instruction Multiple Data optimization potential.

## What is SIMD?

Process multiple samples simultaneously:

```
Scalar: a[0]*b[0], a[1]*b[1], a[2]*b[2], a[3]*b[3]  (4 ops)
SIMD:   a[0:3] * b[0:3]                              (1 op)
```

## Current State

bbx_audio currently uses scalar processing. SIMD optimizations are future work.

## Optimization Targets

### Sample Processing

```rust
// Current (scalar)
for sample in buffer {
    *sample *= gain;
}

// SIMD potential
use std::simd::f32x4;
for chunk in buffer.chunks_exact_mut(4) {
    let v = f32x4::from_slice(chunk);
    let result = v * gain_vec;
    result.copy_to_slice(chunk);
}
```

### Filter Processing

IIR filters can use SIMD for parallel samples:

```rust
// Process 4 independent samples simultaneously
// Requires restructuring state variables
```

## Requirements

### Data Alignment

```rust
#[repr(align(32))]
struct AlignedBuffer {
    data: [f32; 1024],
}
```

### Buffer Size

Buffer size should be multiple of SIMD width:

```
SSE:  4 floats (128-bit)
AVX:  8 floats (256-bit)
AVX-512: 16 floats (512-bit)
```

### Portable SIMD

Use Rust's portable SIMD (nightly):

```rust
#![feature(portable_simd)]
use std::simd::f32x4;
```

Or crates like `wide` for stable Rust.

## Trade-offs

| Aspect | Scalar | SIMD |
|--------|--------|------|
| Complexity | Simple | Complex |
| Portability | Universal | Platform-specific |
| Debugging | Easy | Harder |
| Performance | Baseline | 2-8x faster |
