# Parameter schema fixtures

Shared fixtures pinning the `parameters.*.json` schema across its two
implementations:

- **Rust**: `ParamsFile` / `JsonParamDef` in `bbx_plugin/src/params.rs`
  (tested via `include_str!` in that file's test module)
- **TypeScript**: `ParamDef` / `generateParams()` in `bbx_plugin/client/src/params/`
  (tested by the client's vitest suite)

Both implementations must accept every fixture here, and the order of `id`s in
each file defines the parameter index order on every side (Rust `PARAM_*`
constants, the C++ relay X-macro, and the web `ParameterId` enum). When the
schema changes, update the fixtures and both test suites together.

`effect.json` and `synth.json` mirror `blackboxaudio/template-plugin`'s
parameter files; `edge-cases.json` covers minimal floats (required fields
only), long snake-case ids, two-entry choices, and default-true booleans.
