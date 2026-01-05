# Denormal Prevention

How bbx_audio handles denormal floating-point numbers.

## What are Denormals?

Denormal (subnormal) numbers are very small floats near zero:

```
Normal:   1.0e-38   (exponent > 0)
Denormal: 1.0e-40   (exponent = 0, mantissa != 0)
```

## The Problem

Processing denormals causes severe CPU slowdowns:

- 10-100x slower operations
- Unpredictable latency spikes
- Common in audio (decaying signals)

## Flush Functions

bbx_core provides flush utilities:

```rust
use bbx_core::{flush_denormal_f32, flush_denormal_f64};

let safe = flush_denormal_f32(maybe_denormal);
```

## When They Occur

- Filter feedback paths (decaying)
- Reverb/delay tails
- After gain reduction
- Envelope release phase

## Usage in Blocks

Apply in feedback paths:

```rust
fn process_filter(&mut self, input: f32) -> f32 {
    let output = input + self.state * self.coefficient;
    self.state = flush_denormal_f32(output);
    output
}
```

## Alternative Approaches

### CPU FTZ/DAZ Mode

bbx_core provides the `ftz-daz` Cargo feature for hardware-level denormal prevention. When enabled, the `enable_ftz_daz()` function sets CPU flags to automatically flush denormals to zero.

Enable the feature in your `Cargo.toml`:

```toml
[dependencies]
bbx_core = { version = "...", features = ["ftz-daz"] }

# Or for plugins (recommended):
bbx_plugin = { version = "...", features = ["ftz-daz"] }
```

Usage:

```rust
use bbx_core::denormal::enable_ftz_daz;

// Call once at the start of each audio thread
enable_ftz_daz();
```

When using `bbx_plugin` with the `ftz-daz` feature, this is called automatically during `prepare()`.

| Pros | Cons |
|------|------|
| No per-sample overhead | x86/x86_64 only (no-op on other architectures) |
| Handles all float operations automatically | Affects all code on the thread |
| Recommended for production plugins | |

### DC Offset

Add tiny DC offset to prevent zero crossing:

```rust
const DC_OFFSET: f32 = 1e-25;
let output = (input + DC_OFFSET) * coefficient;
```

Pros: Simple, portable
Cons: Introduces actual DC offset
