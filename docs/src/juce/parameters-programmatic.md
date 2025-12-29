# Programmatic Parameter Definition

Define parameters as Rust `const` arrays for compile-time verification.

## Defining Parameters

Use `ParamDef` constructors to define parameters:

```rust
use bbx_plugin::{ParamDef, ParamType};

const PARAMETERS: &[ParamDef] = &[
    ParamDef::float("GAIN", "Gain", -60.0, 30.0, 0.0),
    ParamDef::float("PAN", "Pan", -100.0, 100.0, 0.0),
    ParamDef::bool("MONO", "Mono", false),
    ParamDef::choice("MODE", "Mode", &["Stereo", "Left", "Right"], 0),
];
```

## ParamDef Constructors

### Boolean

```rust
ParamDef::bool(id, name, default)
```

- `id` - Parameter identifier (e.g., `"MONO"`)
- `name` - Display name (e.g., `"Mono"`)
- `default` - Default value (`true` or `false`)

### Float

```rust
ParamDef::float(id, name, min, max, default)
```

- `id` - Parameter identifier
- `name` - Display name
- `min` - Minimum value
- `max` - Maximum value
- `default` - Default value

### Choice

```rust
ParamDef::choice(id, name, choices, default_index)
```

- `id` - Parameter identifier
- `name` - Display name
- `choices` - Static slice of option labels
- `default_index` - Index of default choice

## ParamType Enum

For more control, use `ParamType` directly:

```rust
use bbx_plugin::{ParamDef, ParamType};

const CUSTOM_PARAM: ParamDef = ParamDef {
    id: "FREQ",
    name: "Frequency",
    param_type: ParamType::Float {
        min: 20.0,
        max: 20000.0,
        default: 440.0,
    },
};
```

## Generating Code

Generate Rust constants:

```rust
use bbx_plugin::generate_rust_indices_from_defs;

let rust_code = generate_rust_indices_from_defs(PARAMETERS);
// Output:
// pub const PARAM_GAIN: usize = 0;
// pub const PARAM_PAN: usize = 1;
// pub const PARAM_MONO: usize = 2;
// pub const PARAM_MODE: usize = 3;
// pub const PARAM_COUNT: usize = 4;
```

Generate C header:

```rust
use bbx_plugin::generate_c_header_from_defs;

let c_header = generate_c_header_from_defs(PARAMETERS);
// Output:
// #define PARAM_GAIN 0
// #define PARAM_PAN 1
// #define PARAM_MONO 2
// #define PARAM_MODE 3
// #define PARAM_COUNT 4
```

## Manual Constants

For simple cases, you can define constants manually:

```rust
// Manual parameter indices
pub const PARAM_GAIN: usize = 0;
pub const PARAM_PAN: usize = 1;
pub const PARAM_MONO: usize = 2;
pub const PARAM_COUNT: usize = 3;
```

And corresponding C header:

```c
#define PARAM_GAIN 0
#define PARAM_PAN 1
#define PARAM_MONO 2
#define PARAM_COUNT 3
```

This approach is simpler but requires manual synchronization between Rust and C++.

## When to Use Programmatic Definition

- Parameters are fixed at compile time
- No need for JSON parsing overhead
- Maximum type safety and compile-time checks
- Simple plugins with few parameters
