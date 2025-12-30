# Parameter System

bbx_audio provides two approaches for defining plugin parameters, both capable of generating code for Rust and C++.

## Overview

Parameters are passed from C++ to Rust as a flat `float` array. You need a consistent way to map array indices to parameter meanings on both sides of the FFI boundary.

```
C++ (JUCE)                    Rust (bbx_plugin)
-----------                   ------------------
params[0] = gainValue    -->  params[PARAM_GAIN]
params[1] = panValue     -->  params[PARAM_PAN]
```

## Two Approaches

### 1. JSON-Based (Recommended)

Define parameters in a `parameters.json` file. This is ideal when:

- Your DAW/framework also reads parameter definitions
- You want a single source of truth
- Parameters are configured at build time

See [parameters.json Format](parameters-json.md) for details.

### 2. Programmatic

Define parameters as Rust `const` arrays. This is ideal when:

- Parameters are known at compile time
- You want maximum compile-time verification
- JSON parsing is unnecessary

See [Programmatic Definition](parameters-programmatic.md) for details.

## Code Generation

Both approaches can generate:

- **Rust constants**: `pub const PARAM_GAIN: usize = 0;`
- **C header defines**: `#define PARAM_GAIN 0`

See [Code Generation](parameters-codegen.md) for integration details.

## Parameter Types

Both approaches support these parameter types:

| Type | Description | Value Range |
|------|-------------|-------------|
| `boolean` | On/off toggle | 0.0 = off, 1.0 = on |
| `float` | Continuous value | min to max |
| `choice` | Discrete options | 0.0, 1.0, 2.0, ... |

## Accessing Parameters in Rust

In your `apply_parameters()` method:

```rust
fn apply_parameters(&mut self, params: &[f32]) {
    // Boolean: compare to 0.5
    self.mono_enabled = params[PARAM_MONO] > 0.5;

    // Float: use directly
    self.gain.level_db = params[PARAM_GAIN];

    // Choice: convert to integer
    let mode = params[PARAM_MODE] as usize;
    self.routing_mode = match mode {
        0 => RoutingMode::Stereo,
        1 => RoutingMode::Left,
        2 => RoutingMode::Right,
        _ => RoutingMode::Stereo,
    };
}
```

## Passing Parameters from JUCE

In your `processBlock()`:

```cpp
// Gather current parameter values
m_paramBuffer[PARAM_GAIN] = *gainParam;
m_paramBuffer[PARAM_PAN] = *panParam;
m_paramBuffer[PARAM_MONO] = *monoParam ? 1.0f : 0.0f;

// Pass to Rust DSP
m_rustDsp.Process(
    inputs, outputs,
    numChannels, numSamples,
    m_paramBuffer.data(),
    static_cast<uint32_t>(m_paramBuffer.size()));
```
