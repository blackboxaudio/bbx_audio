# Error Types

Unified error handling across the bbx_audio workspace.

## Overview

bbx_core provides common error types used throughout the workspace:

- `BbxError` - Main error enum
- `Result<T>` - Type alias for `Result<T, BbxError>`

## BbxError

The main error type:

```rust
use bbx_core::BbxError;

pub enum BbxError {
    /// Generic error with message
    Generic(String),

    /// I/O error wrapper
    Io(std::io::Error),

    /// Invalid parameter value
    InvalidParameter(String),

    /// Resource not found
    NotFound(String),

    /// Operation failed
    OperationFailed(String),

    /// Null pointer in FFI context
    NullPointer,

    /// Invalid buffer size
    InvalidBufferSize,

    /// Graph not prepared for processing
    GraphNotPrepared,

    /// Memory allocation failed
    AllocationFailed,
}
```

## Usage

### Creating Errors

```rust
use bbx_core::BbxError;

fn validate_sample_rate(rate: f64) -> Result<(), BbxError> {
    if rate <= 0.0 {
        return Err(BbxError::InvalidParameter(
            format!("Sample rate must be positive, got {}", rate)
        ));
    }
    Ok(())
}
```

### Using Result Type Alias

```rust
use bbx_core::{BbxError, Result};

fn load_audio(path: &str) -> Result<Vec<f32>> {
    // ... implementation
    Ok(vec![])
}
```

### Error Propagation

```rust
use bbx_core::Result;

fn process() -> Result<()> {
    let audio = load_audio("input.wav")?;  // Propagate errors with ?
    save_audio("output.wav", &audio)?;
    Ok(())
}
```

## FFI Error Codes

For C FFI, errors are represented as integers:

```rust
#[repr(C)]
pub enum BbxErrorCode {
    Ok = 0,
    NullPointer = 1,
    InvalidParameter = 2,
    InvalidBufferSize = 3,
    GraphNotPrepared = 4,
    AllocationFailed = 5,
}
```

Convert between error types:

```rust
use bbx_core::BbxError;

impl From<BbxError> for BbxErrorCode {
    fn from(err: BbxError) -> Self {
        match err {
            BbxError::NullPointer => BbxErrorCode::NullPointer,
            BbxError::InvalidParameter(_) => BbxErrorCode::InvalidParameter,
            BbxError::InvalidBufferSize => BbxErrorCode::InvalidBufferSize,
            BbxError::GraphNotPrepared => BbxErrorCode::GraphNotPrepared,
            BbxError::AllocationFailed => BbxErrorCode::AllocationFailed,
            _ => BbxErrorCode::InvalidParameter,
        }
    }
}
```

## Error Display

`BbxError` implements `Display` for human-readable messages:

```rust
use bbx_core::BbxError;

let err = BbxError::InvalidParameter("sample rate".to_string());
println!("{}", err);  // "Invalid parameter: sample rate"
```

## Integration with std::error::Error

`BbxError` implements `std::error::Error`, allowing integration with standard error handling:

```rust
use bbx_core::BbxError;
use std::error::Error;

fn example() -> std::result::Result<(), Box<dyn Error>> {
    let result = something_that_returns_bbx_result()?;
    Ok(())
}
```
