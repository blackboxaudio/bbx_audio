import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { findPluginRoot } from '../src/params/generate'

describe('findPluginRoot', () => {
    it('walks up to the nearest directory containing plugin.env', () => {
        const root = mkdtempSync(join(tmpdir(), 'bbx-root-test-'))
        writeFileSync(join(root, 'plugin.env'), 'PLUGIN_TYPE=effect\n')
        const nested = join(root, 'web', 'src', 'lib')
        mkdirSync(nested, { recursive: true })

        expect(findPluginRoot(nested)).toBe(root)
        expect(findPluginRoot(root)).toBe(root)
    })

    it('fails with a clear message outside a plugin repo', () => {
        const outside = mkdtempSync(join(tmpdir(), 'bbx-not-a-plugin-'))
        expect(() => findPluginRoot(outside)).toThrow(/No plugin\.env found from .* upward/)
        expect(() => findPluginRoot(outside)).toThrow(/blackboxaudio\/template-plugin/)
    })
})
