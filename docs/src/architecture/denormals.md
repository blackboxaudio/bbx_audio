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

Set CPU flags to auto-flush:

```rust
// Platform-specific, not portable
unsafe { _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON); }
```

Pros: Automatic, no per-sample cost
Cons: Platform-specific, affects all code

### DC Offset

Add tiny DC offset to prevent zero crossing:

```rust
const DC_OFFSET: f32 = 1e-25;
let output = (input + DC_OFFSET) * coefficient;
```

Pros: Simple
Cons: Introduces actual DC offset
