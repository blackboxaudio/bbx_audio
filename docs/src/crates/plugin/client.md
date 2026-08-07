# TypeScript Tooling (@bbx-audio/plugin)

The [`@bbx-audio/plugin`](https://www.npmjs.com/package/@bbx-audio/plugin) npm
package lives at `bbx_plugin/client/` and provides everything a JUCE plugin
derived from
[`blackboxaudio/template-plugin`](https://github.com/blackboxaudio/template-plugin)
needs that is not the plugin itself. It is versioned independently of the
`bbx_*` crates and published automatically when a version bump in its
`package.json` lands on `develop` (the `Publish NPM` workflow).

## The `bbx` CLI

```bash
yarn add --dev @bbx-audio/plugin
```

| Command | Purpose |
| --- | --- |
| `bbx build` | CMake configure/build, macOS codesigning, optional system install |
| `bbx test` | `cargo test` for `lib/dsp` (effect + synth) plus web checks |
| `bbx format` | clang-format + `cargo fmt` over the template layout |
| `bbx tag` | Create or delete the `v<PLUGIN_VERSION>` git tag |
| `bbx generate-params` | Emit parameter IDs (`--cpp <dir>`, `--web`) |

The CLI locates the plugin root by walking up to the nearest `plugin.env` and
fails with a clear message outside a template-derived repo. It has zero runtime
dependencies.

## Library Exports

| Entry | Contents |
| --- | --- |
| `@bbx-audio/plugin` | `generateParams()`, generator functions, schema types |
| `@bbx-audio/plugin/vite` | `bbxParams()` Vite plugin (regenerates the web module in dev) |
| `@bbx-audio/plugin/init` | `window.__BBX__` wiring for visualization data (side effect) |
| `@bbx-audio/plugin/presets` | Typed wrappers over the native preset functions |

## Schema Lockstep

The parameter JSON schema is owned by [`ParamsFile`](./params.md) on the Rust
side and mirrored by the package's TypeScript types. Shared fixtures in
`bbx_plugin/fixtures/params/` are tested by both `cargo test -p bbx_plugin` and
the client's vitest suite.

## See Also

- [Parameter Definitions](./params.md) - The Rust side of the schema
- [Code Generation](../../juce/parameters-codegen.md) - How the layers stay in sync
