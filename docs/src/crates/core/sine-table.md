# Sine Table

A sine wave sampled over one cycle, evaluated at compile time, with a shared interpolating reader that other phase-indexed tables reuse.

## Overview

`SineTable<LENGTH>` stores `LENGTH - 1` cells plus one guard entry that repeats cell zero. The guard lets the last cell interpolate without a wrap check, so the read path is a floor, a multiply, two loads, and one linear interpolation. The cell count must be a power of two.

The storage length is the const parameter because `[f32; N + 1]` is not expressible on stable Rust features. The everyday instance is `SINE_2048` (`SineTable<2049>`, 8 KB, fits in L1 cache).

## API

### Reading

```rust
use bbx_core::SINE_2048;

let sine = SINE_2048.read(0.25);         // sin(2π · 0.25) ≈ 1.0
let cosine = SINE_2048.read_cosine(0.0); // cos, read a quarter cycle ahead
let wrapped = SINE_2048.read(3.25);      // phase wraps with floor: same as 0.25
```

Phase is in cycles (`0 → 1`), matching `PhaseAccumulator` in `bbx_dsp`.

### Building your own size

```rust
use bbx_core::SineTable;

static SMALL: SineTable<1025> = SineTable::new(); // 1024 cells
```

Prefer a `static`: a `const` table is copied wherever it is used by value.

### Shared reader

Any table laid out as cells plus a guard can use the same reader:

```rust
use bbx_core::read_interpolated;

// Four cells of a ramp; the guard holds the final height.
let ramp = [0.0f32, 1.0, 2.0, 3.0, 4.0];
assert_eq!(read_interpolated(&ramp, 0.125), 0.5);
```

For a non-periodic table the guard holds the final height rather than a repeat of cell zero.

## Accuracy

Linear interpolation with step `h = 2π / N` has error bounded by `(2π / N)² / 8`:

| Cells | Bound  | Level   |
|-------|--------|---------|
| 1024  | 4.7e-6 | −106 dB |
| 2048  | 1.2e-6 | −118 dB |
| 4096  | 3.0e-7 | −130 dB |

Entries are computed in `f64` from a Taylor series on a quarter cycle (terms through `x¹¹ / 11!`) and rounded to `f32`, so each sits within `1e-6` of the true sine.

## Realtime Safety

`read` allocates nothing and has no branches beyond the clamp that keeps the index in range. The table itself is `no_std` compatible and lives in read-only data when declared as a `static`.
