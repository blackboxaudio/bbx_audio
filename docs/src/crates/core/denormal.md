# Denormal Handling

Utilities for flushing denormal (subnormal) floating-point values to zero.

## Why Denormals Matter

Denormal numbers are very small floating-point values near zero. Processing them can cause significant CPU slowdowns (10-100x slower) on many processors.

In audio processing, denormals commonly occur:
- In filter feedback paths as signals decay
- After gain reduction to very low levels
- In reverb/delay tail-off

## API

### flush_denormal_f32

Flush a 32-bit float to zero if denormal:

```rust
use bbx_core::flush_denormal_f32;

let small_value = 1.0e-40_f32;  // Denormal
let result = flush_denormal_f32(small_value);
assert_eq!(result, 0.0);

let normal_value = 0.5_f32;
let result = flush_denormal_f32(normal_value);
assert_eq!(result, 0.5);
```

### flush_denormal_f64

Flush a 64-bit float to zero if denormal:

```rust
use bbx_core::flush_denormal_f64;

let small_value = 1.0e-310_f64;  // Denormal
let result = flush_denormal_f64(small_value);
assert_eq!(result, 0.0);
```

## Usage Patterns

### In Feedback Loops

```rust
use bbx_core::flush_denormal_f32;

fn process_filter(input: f32, state: &mut f32, coefficient: f32) -> f32 {
    let output = input + *state * coefficient;
    *state = flush_denormal_f32(output);  // Prevent denormal accumulation
    output
}
```

### Block Processing

```rust
use bbx_core::flush_denormal_f32;

fn process_block(samples: &mut [f32]) {
    for sample in samples {
        *sample = flush_denormal_f32(*sample);
    }
}
```

## Performance

The flush functions use bit manipulation, not branching:

```rust
// Conceptually (actual implementation may differ):
fn flush_denormal_f32(x: f32) -> f32 {
    let bits = x.to_bits();
    let exponent = (bits >> 23) & 0xFF;
    if exponent == 0 && (bits & 0x007FFFFF) != 0 {
        0.0  // Denormal
    } else {
        x    // Normal
    }
}
```

This is typically faster than relying on FPU denormal handling.

## Hardware FTZ/DAZ Mode

For maximum performance, bbx_core provides the `ftz-daz` Cargo feature that enables hardware-level denormal handling on x86/x86_64 processors.

### Enabling the Feature

```toml
[dependencies]
bbx_core = { version = "...", features = ["ftz-daz"] }
```

### enable_ftz_daz

When the feature is enabled, call `enable_ftz_daz()` once at the start of each audio processing thread:

```rust
use bbx_core::denormal::enable_ftz_daz;

fn audio_thread_init() {
    enable_ftz_daz();
    // All subsequent float operations on this thread will auto-flush denormals
}
```

This sets two CPU flags:
- **FTZ (Flush-To-Zero)**: Denormal results are flushed to zero
- **DAZ (Denormals-Are-Zero)**: Denormal inputs are treated as zero

### Platform Support

- **x86/x86_64**: Full support via MXCSR register
- **Other architectures**: No-op (use the software flush functions instead)

### bbx_plugin Integration

When using `bbx_plugin` with the `ftz-daz` feature enabled, `enable_ftz_daz()` is called automatically during `prepare()`:

```toml
[dependencies]
bbx_plugin = { version = "...", features = ["ftz-daz"] }
```

This is the recommended approach for audio plugins.

## Software vs Hardware Approach

| Approach | Use Case |
|----------|----------|
| `flush_denormal_*` functions | Cross-platform, targeted flushing in specific code paths |
| `enable_ftz_daz()` | Maximum performance on x86/x86_64, affects all operations |

For production audio plugins on desktop platforms, enabling the `ftz-daz` feature is recommended. The software flush functions remain useful for cross-platform code or when you need fine-grained control over which values are flushed.
