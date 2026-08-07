# @bbx-audio/plugin

Shared tooling for JUCE plugins derived from
[`blackboxaudio/template-plugin`](https://github.com/blackboxaudio/template-plugin):
the `bbx` CLI, parameter codegen, and WebView glue. The package mirrors the
`bbx_plugin` Rust crate the way `@bbx-audio/net` mirrors `bbx_net`, and sits
alongside `@bbx-audio/nectar` (JUCE bridge) and `@bbx-audio/honey` (components)
in the plugin stack.

## Installation

```bash
yarn add --dev @bbx-audio/plugin
```

```bash
npm install --save-dev @bbx-audio/plugin
```

Installing sibling `@bbx-audio` packages that are restricted (honey) requires
`BBX_AUDIO_NPM_READ_TOKEN` to be exported (see the template's README).

## The `bbx` CLI

All commands must run inside a plugin repo derived from the template — the CLI
walks up from the current directory to the nearest `plugin.env` and fails with
a clear message when there is none. Plugins alias the commands in their
`package.json` scripts and extend them through script composition:

```json
{
    "scripts": {
        "build": "bbx build",
        "test": "bbx test && node scripts/test-extras.mjs"
    }
}
```

| Command | Purpose |
| --- | --- |
| `bbx build` | CMake configure + build, macOS codesigning, optional install copy (`-b Debug`, `-w`, `-d`, `-c`, `-r`, `-s`, `-j N`) |
| `bbx test` | `cargo test` for `lib/dsp` (effect + synth) and web checks (`-r`, `-w`) |
| `bbx format` | clang-format over `src/` + `lib/cortex/src/`, `cargo fmt` for `lib/dsp` |
| `bbx tag` | Create/push (or `-d` delete) the `v<PLUGIN_VERSION>` git tag |
| `bbx generate-params` | Emit parameter IDs (`--cpp <dir>`, `--web`) |

Run `bbx <command> --help` for full options. macOS codesigning reads
`APPLE_DEVELOPER_ID_APP` from the plugin's gitignored `.env`.

## Parameter codegen

`parameters.*.json` is the single source of truth for plugin parameters; the
schema is owned by the `bbx_plugin` crate (`ParamsFile`) and pinned by shared
fixtures in `bbx_plugin/fixtures/params/`.

```ts
import { generateParams } from '@bbx-audio/plugin'

generateParams({ cppOutDir: 'bin/generated', web: true })
```

- `--cpp` emits `bbx_generated_params.h`: JUCE parameter ID constants plus the
  `BBX_FOR_EACH_PARAM_RELAY` X-macro used by the WebView relay setup.
- `--web` emits `web/src/lib/generated/params.ts`: the `ParameterId` enum and
  the active `pluginType`.

### Vite plugin

```ts
// web/vite.config.ts
import { bbxParams } from '@bbx-audio/plugin/vite'

export default defineConfig({
    plugins: [bbxParams(), /* ... */],
})
```

Regenerates the web module on config load and whenever `parameters.*.json` or
`plugin.env` change while the dev server runs.

## WebView glue

```ts
// web/src/main.ts
import '@bbx-audio/plugin/init'
```

Wires `window.__BBX__` (sample/spectrum/VU updates from the C++ backend) to the
honey stores and augments the `Window` type. Requires `@bbx-audio/nectar` and
`@bbx-audio/honey` as dependencies of the consuming plugin.

```ts
import { listPresets, loadPreset, savePreset } from '@bbx-audio/plugin/presets'
```

Typed wrappers over the template's native preset functions
(`cortex::PresetManager` on the C++ side).

## Development

```bash
yarn install    # requires BBX_AUDIO_NPM_READ_TOKEN
yarn build      # tsc declarations + vite lib build
yarn test       # vitest (shares fixtures with the Rust ParamsFile tests)
yarn lint:check && yarn format:check
```

## Versioning & Compatibility

This package is versioned independently of the `bbx_plugin` Rust crate. The
table below tracks which package versions target which crate's parameter/preset
interfaces (the `params.json` schema and generated FFI surface); keep it current
when those interfaces change.

| `@bbx-audio/plugin` | `bbx_plugin` |
| ------------------- | ------------ |
| 0.5.x               | 0.5.x        |

## License

MIT
