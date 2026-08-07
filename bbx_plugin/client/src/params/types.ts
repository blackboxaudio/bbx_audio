/**
 * Parameter JSON schema types.
 *
 * These mirror `JsonParamDef` in `bbx_plugin/src/params.rs` — the Rust side owns
 * the schema. Shared fixtures under `bbx_plugin/fixtures/params/` are tested
 * against both implementations to keep them in lockstep.
 */

export type PluginType = 'effect' | 'synth'

export type ParamType = 'float' | 'choice' | 'boolean'

export interface ParamDef {
    id: string
    name: string
    type: ParamType
    defaultValue?: number | boolean
    defaultValueIndex?: number
    min?: number
    max?: number
    unit?: string
    midpoint?: number
    interval?: number
    fractionDigits?: number
    choices?: string[]
}

export interface ParamsFile {
    parameters: ParamDef[]
}
