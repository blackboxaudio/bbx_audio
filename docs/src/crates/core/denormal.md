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

## Alternatives

### CPU Flush-to-Zero Mode

Some CPUs support a "flush-to-zero" (FTZ) mode that automatically flushes denormals. This is faster but:

- Platform-specific
- Requires changing FPU state
- May affect other code running on the same thread

### DAZ (Denormals-Are-Zero)

Similar to FTZ but for inputs. Also platform-specific.

The explicit flush functions work everywhere and have predictable behavior.
