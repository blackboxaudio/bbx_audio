# parameters.json Format

Define plugin parameters in a JSON file for cross-language code generation.

## File Structure

```json
{
  "parameters": [
    {
      "id": "GAIN",
      "name": "Gain",
      "type": "float",
      "min": -60.0,
      "max": 30.0,
      "defaultValue": 0.0,
      "unit": "dB"
    },
    {
      "id": "PAN",
      "name": "Pan",
      "type": "float",
      "min": -100.0,
      "max": 100.0,
      "defaultValue": 0.0
    },
    {
      "id": "MONO",
      "name": "Mono",
      "type": "boolean",
      "defaultValue": false
    },
    {
      "id": "MODE",
      "name": "Routing Mode",
      "type": "choice",
      "choices": ["Stereo", "Left", "Right", "Swap"],
      "defaultValueIndex": 0
    }
  ]
}
```

## Field Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Parameter identifier (uppercase, used in code generation) |
| `name` | string | Display name for UI |
| `type` | string | `"boolean"`, `"float"`, or `"choice"` |

### Boolean Parameters

| Field | Type | Description |
|-------|------|-------------|
| `defaultValue` | boolean | Default state (`true` or `false`) |

### Float Parameters

| Field | Type | Description |
|-------|------|-------------|
| `min` | number | Minimum value |
| `max` | number | Maximum value |
| `defaultValue` | number | Default value |
| `unit` | string | Optional unit label (e.g., "dB", "Hz", "%") |
| `midpoint` | number | Optional midpoint for skewed ranges |
| `interval` | number | Optional step interval |
| `fractionDigits` | integer | Optional decimal places to display |

### Choice Parameters

| Field | Type | Description |
|-------|------|-------------|
| `choices` | string[] | Array of option labels |
| `defaultValueIndex` | integer | Index of default choice (0-based) |

## Complete Example

```json
{
  "parameters": [
    {
      "id": "INVERT_LEFT",
      "name": "Invert Left",
      "type": "boolean",
      "defaultValue": false
    },
    {
      "id": "INVERT_RIGHT",
      "name": "Invert Right",
      "type": "boolean",
      "defaultValue": false
    },
    {
      "id": "CHANNEL_MODE",
      "name": "Channel Mode",
      "type": "choice",
      "choices": ["Stereo", "Left", "Right", "Swap"],
      "defaultValueIndex": 0
    },
    {
      "id": "MONO",
      "name": "Sum to Mono",
      "type": "boolean",
      "defaultValue": false
    },
    {
      "id": "GAIN",
      "name": "Gain",
      "type": "float",
      "min": -60.0,
      "max": 30.0,
      "defaultValue": 0.0,
      "unit": "dB",
      "fractionDigits": 1
    },
    {
      "id": "PAN",
      "name": "Pan",
      "type": "float",
      "min": -100.0,
      "max": 100.0,
      "defaultValue": 0.0,
      "interval": 1.0
    },
    {
      "id": "DC_OFFSET",
      "name": "DC Offset Removal",
      "type": "boolean",
      "defaultValue": false
    }
  ]
}
```

## Parsing in Rust

```rust
use bbx_plugin::ParamsFile;

fn load_parameters() -> ParamsFile {
    let json = include_str!("../parameters.json");
    ParamsFile::from_json(json).expect("Failed to parse parameters.json")
}
```

## Generated Output

From the above JSON, code generation produces:

**Rust:**
```rust
pub const PARAM_INVERT_LEFT: usize = 0;
pub const PARAM_INVERT_RIGHT: usize = 1;
pub const PARAM_CHANNEL_MODE: usize = 2;
pub const PARAM_MONO: usize = 3;
pub const PARAM_GAIN: usize = 4;
pub const PARAM_PAN: usize = 5;
pub const PARAM_DC_OFFSET: usize = 6;
pub const PARAM_COUNT: usize = 7;
```

**C Header:**
```c
#define PARAM_INVERT_LEFT 0
#define PARAM_INVERT_RIGHT 1
#define PARAM_CHANNEL_MODE 2
#define PARAM_MONO 3
#define PARAM_GAIN 4
#define PARAM_PAN 5
#define PARAM_DC_OFFSET 6
#define PARAM_COUNT 7
```
