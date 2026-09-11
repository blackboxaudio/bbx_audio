# Release Process

How to release new versions of bbx_audio. This page summarizes the flow; the
authoritative, step-by-step guide is `RELEASING.md` in the repository root.

## Prerequisites

- Push access to the repository
- `CARGO_REGISTRY_TOKEN` (crates.io) and `NPM_TOKEN` (npm) configured in GitHub secrets
- CI passing on `develop`

## Versioning

All crates use **lockstep versioning** — they share a single version via
`[workspace.package]` in the root `Cargo.toml`. Bump the version in one place (plus the
internal versions under `[workspace.dependencies]`), not in each crate individually.

The npm client packages (`@bbx-audio/net` in `bbx_net/client/`, `@bbx-audio/plugin` in
`bbx_plugin/client/`) are **versioned independently** of the crates and are not part of
the crate release train — see [NPM Client Packages](#npm-client-packages) below.

## Releasing

Releases are triggered by **merging a `release/v*` branch into `develop`** — not by pushing
a tag manually.

1. Create a release branch from `develop`: `release/vX.Y.Z`
2. Bump the version in the root `Cargo.toml` (`[workspace.package].version` and the
   `[workspace.dependencies]` entries), update the `README.md` version badge, and update
   `CHANGELOG.md`.
3. Open a PR `release/vX.Y.Z` → `develop`, let CI pass, and merge it.

Merging the release PR runs the `Release` workflow (`.github/workflows/cd.release.yml`),
which automatically:

1. **Creates the tag** — extracts `vX.Y.Z` from the branch name, verifies it matches
   `Cargo.toml`, and pushes the tag.
2. **Opens a `develop → main` sync PR** (merge it afterward to bring `main` up to date).
3. **Validates** — checks out the tag and runs `cargo test --workspace --release`.
4. **Publishes to crates.io** in dependency order (see below).
5. **Creates a GitHub Release** with the changelog section for the version.

A manual `workflow_dispatch` run is also available; it publishes from an already-existing
tag whose version matches `Cargo.toml`.

## Crate Publishing Order

Crates publish in dependency order, with index-propagation delays after `bbx_core` and
`bbx_dsp`:

1. `bbx_core` (no internal dependencies)
2. `bbx_midi`
3. `bbx_net`
4. `bbx_dsp` (depends on `bbx_core`)
5. `bbx_file` (depends on `bbx_dsp`)
6. `bbx_player` (depends on `bbx_core`, `bbx_dsp`)
7. `bbx_plugin` (depends on `bbx_dsp`)
8. `bbx_draw` (depends on `bbx_dsp`)
9. `bbx_daisy` (depends on `bbx_core`, `bbx_dsp`)

`bbx_sandbox` is not published (`publish = false`).

## Manual Publishing

If the automated publish fails partway through, do **not** re-run the whole workflow (it
will fail on already-published crates). Instead, publish the remaining crates in the order
above:

```bash
cargo publish -p <crate_name>
# Wait ~30-60s after bbx_core / bbx_dsp for the crates.io index to update
```

See `RELEASING.md` in the repository root for the detailed procedure and troubleshooting.

## NPM Client Packages

`@bbx-audio/net` and `@bbx-audio/plugin` follow the same model as `@bbx-audio/honey` and
`@bbx-audio/nectar`: bump the `version` in the client's `package.json` as part of a normal
PR, and when it lands on `develop` the `Publish NPM` workflow
(`.github/workflows/cd.npm.yml`) builds and publishes the package, then pushes a
`net-vX.Y.Z` / `plugin-vX.Y.Z` tag. Merges without a version bump publish nothing.

Client and crate versions no longer need to match. When a crate change affects something a
client depends on (the `bbx_net` WebSocket protocol, `bbx_plugin` parameter codegen),
update the client in the same PR, bump its version, and keep the compatibility notes in
the client READMEs accurate.
