# Release Process

How to release new versions of bbx_audio.

## Prerequisites

- Push access to the repository
- `CARGO_REGISTRY_TOKEN` configured in GitHub secrets

## Version Bump

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Commit changes

```bash
# Example: bumping to 0.2.0
# Edit Cargo.toml files...
git add -A
git commit -m "Bump version to 0.2.0"
```

## Creating a Release

1. Create and push a version tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

2. GitHub Actions will:
   - Run tests
   - Publish to crates.io
   - Create GitHub release

## Crate Publishing Order

Crates must be published in dependency order:

1. `bbx_core` (no dependencies)
2. `bbx_midi` (no internal dependencies)
3. `bbx_dsp` (depends on bbx_core)
4. `bbx_file` (depends on bbx_dsp)
5. `bbx_plugin` (depends on bbx_dsp)

## Manual Publishing

If needed:

```bash
cargo publish -p bbx_core
# Wait for crates.io index update (~1 minute)
cargo publish -p bbx_midi
cargo publish -p bbx_dsp
cargo publish -p bbx_file
cargo publish -p bbx_plugin
```

## Troubleshooting

### Publish Failure

If publishing fails mid-way:

1. Fix the issue
2. Bump patch version
3. Retry publishing remaining crates

### Version Conflicts

Ensure all workspace crates use the same version in dependencies.

See `RELEASING.md` in the repository root for detailed instructions.
