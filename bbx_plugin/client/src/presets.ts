import { getNativeFunction } from '@bbx-audio/nectar'

// Thin wrappers over the C++ PresetManager native functions registered by the
// template's WebView backend (cortex::PresetManager). Hook point for a
// plugin-specific preset browser UI.

export async function listPresets(): Promise<string[]> {
    return getNativeFunction<string[]>('listPresets')()
}

export async function savePreset(name: string): Promise<boolean> {
    return getNativeFunction<boolean>('savePreset')(name)
}

export async function loadPreset(name: string): Promise<boolean> {
    return getNativeFunction<boolean>('loadPreset')(name)
}
