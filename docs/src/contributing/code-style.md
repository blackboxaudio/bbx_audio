# Code Style

Coding conventions for bbx_audio.

## Formatting

Use nightly rustfmt:

```bash
cargo +nightly fmt
```

Configuration is in `rustfmt.toml`.

## Linting

Use nightly clippy:

```bash
cargo +nightly clippy
```

Fix all warnings before submitting.

## Naming Conventions

### Types

- **Structs**: PascalCase (`OscillatorBlock`)
- **Enums**: PascalCase (`Waveform`)
- **Traits**: PascalCase (`Block`, `Sample`)

### Functions and Methods

- **Functions**: snake_case (`process_buffers`)
- **Methods**: snake_case (`add_oscillator`)
- **Constructors**: `new()` or `from_*()` / `with_*()`

### Variables

- **Local**: snake_case (`buffer_size`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_BLOCK_INPUTS`)

## Documentation

### Public Items

All public items must have documentation:

```rust
/// A block that generates waveforms.
///
/// # Example
///
/// ```
/// let osc = OscillatorBlock::new(440.0, Waveform::Sine);
/// ```
pub struct OscillatorBlock<S: Sample> {
    // ...
}
```

### Module Documentation

Each module should have a top-level doc comment:

```rust
//! DSP graph system.
//!
//! This module provides [`Graph`] for managing connected DSP blocks.
```

## Safety Comments

All unsafe blocks must have a `SAFETY:` comment:

```rust
// SAFETY: The buffer indices are pre-computed and validated
// during prepare_for_playback(), guaranteeing valid access.
unsafe {
    let slice = std::slice::from_raw_parts(ptr, len);
}
```

## Error Handling

- Use `Result<T, BbxError>` for fallible operations
- Use `Option<T>` for optional values
- Avoid `unwrap()` in library code
- Use `expect()` with clear messages for invariants

## Testing

- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Document test coverage for new features
