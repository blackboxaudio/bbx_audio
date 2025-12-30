# Migration Guide

Upgrading between bbx_audio versions.

## 0.1.x to 0.2.x

*(Placeholder for future breaking changes)*

### API Changes

When breaking changes occur, they will be documented here with:

- What changed
- Why it changed
- How to update your code

### Example Migration

```rust
// Old API (0.1.x)
let graph = GraphBuilder::new(44100.0, 512, 2).build();

// New API (0.2.x) - hypothetical
let graph = GraphBuilder::new()
    .sample_rate(44100.0)
    .buffer_size(512)
    .channels(2)
    .build();
```

## General Upgrade Process

1. **Read the changelog** - Understand what changed
2. **Update dependencies** - Bump version in `Cargo.toml`
3. **Run tests** - Identify breaking changes
4. **Fix compilation errors** - Update API calls
5. **Test thoroughly** - Verify audio output

## Deprecation Policy

- Deprecated APIs are marked with `#[deprecated]`
- Deprecated APIs remain for at least one minor version
- Removal happens in the next major version

## Getting Help

If you encounter migration issues:

1. Check the [changelog](changelog.md)
2. Search [GitHub issues](https://github.com/blackboxaudio/bbx_audio/issues)
3. Open a new issue if needed
