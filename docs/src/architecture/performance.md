# Performance Considerations

Optimizing DSP performance in bbx_audio.

## Key Metrics

| Metric | Target |
|--------|--------|
| Latency | < 1 buffer |
| CPU usage | < 50% of budget |
| Memory | Predictable, fixed |
| Allocation | Zero during process |

## Optimization Strategies

### Pre-computation

Calculate once, use many times:

```rust
// In prepare()
self.coefficient = calculate_filter_coeff(self.cutoff, context.sample_rate);

// In process() - just use it
output = input * self.coefficient + self.state;
```

### Cache Efficiency

Keep hot data together:

```rust
// Good: Contiguous buffer storage
audio_buffers: Vec<AudioBuffer<S>>

// Good: Sequential processing
for block_id in &self.execution_order {
    self.process_block(*block_id);
}
```

### Branch Avoidance

Prefer branchless code:

```rust
// Avoid
if condition {
    result = a;
} else {
    result = b;
}

// Prefer
let mask = condition as f32;  // 0.0 or 1.0
result = a * mask + b * (1.0 - mask);
```

### SIMD Potential

Design for SIMD:

- Process 4+ samples at once
- Align buffers to 16/32 bytes
- Avoid data-dependent branches

## Profiling

Measure before optimizing:

```rust
#[cfg(feature = "profiling")]
let _span = tracing::span!(tracing::Level::TRACE, "process_block");
```

## Common Bottlenecks

1. **Memory allocation** in audio thread
2. **Cache misses** from scattered data
3. **Branch misprediction** in complex logic
4. **Function call overhead** for tiny operations
5. **Denormal processing** in filter feedback
