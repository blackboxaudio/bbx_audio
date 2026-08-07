import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

import { generateCppHeader, generateParams, generateTsModule } from '../src/params/generate'
import type { ParamsFile } from '../src/params/types'

// Shared with the Rust ParamsFile tests (see ../../fixtures/params/README.md):
// both implementations must accept these files, in this order.
function loadFixture(name: string): ParamsFile {
    const path = fileURLToPath(new URL(`../../fixtures/params/${name}`, import.meta.url))
    return JSON.parse(readFileSync(path, 'utf8')) as ParamsFile
}

const effect = loadFixture('effect.json')
const synth = loadFixture('synth.json')
const edgeCases = loadFixture('edge-cases.json')

describe('fixtures', () => {
    it('preserves effect fixture id order', () => {
        expect(effect.parameters.map((p) => p.id)).toEqual([
            'INVERT_LEFT_CHANNEL',
            'INVERT_RIGHT_CHANNEL',
            'CHANNEL_CONFIGURATION',
            'MONO',
            'GAIN',
            'PAN',
            'DC_OFFSET',
        ])
    })

    it('preserves synth fixture id order', () => {
        expect(synth.parameters.map((p) => p.id)).toEqual([
            'OSC_TYPE',
            'ATTACK',
            'DECAY',
            'SUSTAIN',
            'RELEASE',
            'FILTER_CUTOFF',
            'FILTER_RESONANCE',
            'MASTER_GAIN',
        ])
    })

    it('accepts the edge-case shapes', () => {
        expect(edgeCases.parameters).toHaveLength(4)
        expect(edgeCases.parameters[0]).toEqual({ id: 'DRIVE', name: 'Drive', type: 'float' })
        expect(edgeCases.parameters[3].defaultValue).toBe(true)
    })
})

describe('generateCppHeader', () => {
    it('emits the parameter ids and relay X-macro in fixture order', () => {
        const header = generateCppHeader(effect.parameters)
        expect(header).toMatchSnapshot()
    })

    it('maps each parameter type to its JUCE relay pair', () => {
        const header = generateCppHeader(edgeCases.parameters)
        expect(header).toContain('X(drive, DRIVE_PARAM_ID, juce::WebSliderRelay, juce::WebSliderParameterAttachment)')
        expect(header).toContain('X(mode, MODE_PARAM_ID, juce::WebComboBoxRelay, juce::WebComboBoxParameterAttachment)')
        expect(header).toContain(
            'X(bypass, BYPASS_PARAM_ID, juce::WebToggleButtonRelay, juce::WebToggleButtonParameterAttachment)'
        )
    })

    it('rejects unknown parameter types with a named error', () => {
        const bad = [{ id: 'GAIN', name: 'Gain', type: 'flaot' }] as unknown as ParamsFile['parameters']
        expect(() => generateCppHeader(bad)).toThrow('Unknown parameter type "flaot" for "GAIN"')
    })
})

describe('generateTsModule', () => {
    it('merges effect and synth sections with dedupe', () => {
        const module = generateTsModule(effect.parameters, synth.parameters, 'synth')
        expect(module).toMatchSnapshot()
    })

    it('keeps enum entries in fixture order', () => {
        const module = generateTsModule(effect.parameters, synth.parameters, 'effect')
        const entries = [...module.matchAll(/^ {4}(\w+) = '(\w+)',$/gm)].map((match) => match[2])
        expect(entries).toEqual([...effect.parameters.map((p) => p.id), ...synth.parameters.map((p) => p.id)])
    })
})

describe('generateParams', () => {
    function makePluginRepo(options: { synthFile?: boolean } = {}): string {
        const root = mkdtempSync(join(tmpdir(), 'bbx-plugin-test-'))
        writeFileSync(join(root, 'plugin.env'), 'PLUGIN_NAME=Test\nPLUGIN_TYPE=effect\n')
        writeFileSync(join(root, 'parameters.effect.json'), JSON.stringify(effect))
        if (options.synthFile) {
            writeFileSync(join(root, 'parameters.synth.json'), JSON.stringify(synth))
        }
        return root
    }

    it('emits the C++ header and web module', () => {
        const root = makePluginRepo({ synthFile: true })
        const cppOutDir = join(root, 'generated')
        expect(generateParams({ repoRoot: root, cppOutDir, web: true })).toBe('effect')

        const header = readFileSync(join(cppOutDir, 'bbx_generated_params.h'), 'utf8')
        expect(header).toContain('inline const juce::String GAIN_PARAM_ID = "GAIN";')

        const module = readFileSync(join(root, 'web', 'src', 'lib', 'generated', 'params.ts'), 'utf8')
        expect(module).toContain("Gain = 'GAIN',")
        expect(module).toContain("OscType = 'OSC_TYPE',")
        expect(module).toContain("export const pluginType: 'synth' | 'effect' = 'effect'")
    })

    it('tolerates a missing counterpart parameters file for the web module', () => {
        const root = makePluginRepo({ synthFile: false })
        expect(generateParams({ repoRoot: root, web: true })).toBe('effect')

        const module = readFileSync(join(root, 'web', 'src', 'lib', 'generated', 'params.ts'), 'utf8')
        expect(module).toContain("Gain = 'GAIN',")
        expect(module).not.toContain('OSC_TYPE')
    })

    it('respects a custom web output path', () => {
        const root = makePluginRepo()
        const webOutFile = join(root, 'custom', 'params.ts')
        generateParams({ repoRoot: root, web: true, webOutFile })
        expect(readFileSync(webOutFile, 'utf8')).toContain('export enum ParameterId')
    })
})
