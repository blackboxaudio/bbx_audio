# Release Process

This document describes how to release new versions of bbx_audio crates.

## Prerequisites

1. Ensure you have push access to the repository
2. Ensure `CARGO_REGISTRY_TOKEN` secret is configured in GitHub repository settings
3. All CI checks must be passing on the `develop` branch

## Version Bump Procedure

All crates use lockstep versioning - they share the same version number.

### 1. Create Release Branch

```bash
git checkout develop
git pull origin develop
git checkout -b release/v0.2.0
```

### 2. Update Version Numbers

Update the version in the root `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Update this

[workspace.dependencies]
bbx_core = { version = "0.2.0", path = "bbx_core" }  # Update all internal deps
bbx_dsp = { version = "0.2.0", path = "bbx_dsp" }
bbx_file = { version = "0.2.0", path = "bbx_file" }
bbx_midi = { version = "0.2.0", path = "bbx_midi" }
bbx_net = { version = "0.2.0", path = "bbx_net" }
bbx_player = { version = "0.2.0", path = "bbx_player" }
bbx_plugin = { version = "0.2.0", path = "bbx_plugin" }
```

Update the version in the root `README.md`:

```markdown
[![Version: v0.2.0](https://img.shields.io/badge/Version-v0.2.0-blue.svg)](https://github.com/blackboxaudio/bbx_audio)
```

Manually scan through the `docs/` directory and other documentation files to ensure all version references are updated accordingly.

### 3. Update Changelog

Generate changelog entries:

```bash
# Install git-cliff if needed
cargo install git-cliff

# Generate changelog for new version
git cliff --unreleased --tag v0.2.0 --prepend CHANGELOG.md
```

Or manually add entries to `CHANGELOG.md`.

### 4. Verify Publishing Works

```bash
# Run dry-run for all crates
for crate in bbx_core bbx_midi bbx_net bbx_dsp bbx_file bbx_player bbx_plugin bbx_draw; do
    cargo publish --dry-run -p $crate
done
```

### 5. Commit and Push

```bash
git add .
git commit -m "chore(release): prepare v0.2.0"
git push origin release/v0.2.0
```

### 6. Create Pull Request to Develop

1. Create PR: `release/v0.2.0` -> `develop`
2. Title: `chore(release): v0.2.0`
3. Wait for CI to pass
4. Get approval and merge (squash merge)

### 7. Automated: Tag Creation

When the release PR is merged to `develop`, the CI automatically:

1. Extracts the version from the branch name (`release/v0.2.0` → `v0.2.0`)
2. Verifies the version matches `Cargo.toml`
3. Creates and pushes the git tag

This triggers the publish workflow.

### 8. Automated: Sync Main PR

The CI also automatically creates a PR to sync `develop` → `main`:

1. PR is created with title: `chore: sync main with v0.2.0 release`
2. Review and merge the PR (can be fast-forward if main hasn't diverged)

## What Happens Automatically

When a `release/v*` PR is merged to `develop`:

1. **Create tag job** extracts version, verifies Cargo.toml, creates and pushes the tag
2. **Sync main job** creates a PR from `develop` to `main`

When the tag is pushed:

1. **Validate job** runs tests and verifies version
2. **Publish job** publishes crates to crates.io in dependency order
3. **GitHub Release** is created with auto-generated changelog

## Troubleshooting

### Publish Failed Mid-Way

If publishing fails after some crates are published:

1. Do NOT re-run the workflow (will fail on already-published crates)
2. Manually publish remaining crates in order:
   ```bash
   cargo publish -p <crate_name>
   ```

### Version Already Exists on crates.io

You cannot overwrite a published version. You must:

1. Bump to a new patch version (e.g., 0.3.0)
2. Create a new tag

### crates.io Index Propagation

If dependent crate publish fails with "dependency not found":

- Wait 60 seconds and retry
- The workflow includes 30-second delays, but crates.io can be slow

## GitHub Repository Setup

### Required Secrets

Add to GitHub repository Settings > Secrets and variables > Actions:

| Secret Name | Description |
|-------------|-------------|
| `CARGO_REGISTRY_TOKEN` | crates.io API token with publish scope |

### Creating a crates.io API Token

1. Go to https://crates.io/settings/tokens
2. Click "New Token"
3. Name it (e.g., "bbx_audio GitHub Actions")
4. Select scopes: `publish-new` and `publish-update`
5. Copy the token and add it as `CARGO_REGISTRY_TOKEN` secret in GitHub

### Optional: Deployment Environment

Create a `crates-io` environment in GitHub with:

- Required reviewers for production releases
- Deployment branches limited to `main`
