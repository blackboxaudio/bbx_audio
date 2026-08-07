import { describe, expect, it } from 'vitest'

import { normalizeArgs } from '../src/cli/args'
import { buildCmakeConfigureArgs, resolveBuildOptions } from '../src/cli/commands/build'

describe('normalizeArgs', () => {
    it('splits legacy short-with-equals flags', () => {
        expect(normalizeArgs(['-b=Debug', '-w'])).toEqual(['-b', 'Debug', '-w'])
        expect(normalizeArgs(['-j=4'])).toEqual(['-j', '4'])
    })

    it('leaves long-form and plain flags untouched', () => {
        expect(normalizeArgs(['--build-type=Debug', '-b', 'Debug', '--copy'])).toEqual([
            '--build-type=Debug',
            '-b',
            'Debug',
            '--copy',
        ])
    })
})

describe('resolveBuildOptions', () => {
    it.each([[['-b=Debug']], [['-b', 'Debug']], [['--build-type=Debug']], [['--build-type', 'Debug']]])(
        'accepts %j for a Debug build',
        (argv) => {
            const options = resolveBuildOptions(argv)
            expect(options.buildType).toBe('Debug')
            expect(options.logging).toBe('ON')
            expect(buildCmakeConfigureArgs(options)).toContain('-DCMAKE_BUILD_TYPE=Debug')
            expect(buildCmakeConfigureArgs(options)).toContain('-DBBX_ENABLE_LOGGING=ON')
        }
    )

    it('defaults to a Release build without logging', () => {
        const options = resolveBuildOptions([])
        expect(options.buildType).toBe('Release')
        expect(options.logging).toBe('OFF')
        expect(options.devServer).toBe('OFF')
        expect(options.copy).toBe(false)
        expect(options.parallel).toBeGreaterThan(0)
        expect(buildCmakeConfigureArgs(options)).toEqual([
            '-S',
            '.',
            '-B',
            'bin',
            '-DCMAKE_BUILD_TYPE=Release',
            '-DBBX_USE_DEV_SERVER=OFF',
            '-DBBX_ENABLE_LOGGING=OFF',
        ])
    })

    it('maps the boolean flags', () => {
        const options = resolveBuildOptions(['-c', '-d', '-r', '-s', '-w'])
        expect(options.copy).toBe(true)
        expect(options.devServer).toBe('ON')
        expect(options.removePrev).toBe(true)
        expect(options.skipSign).toBe(true)
        expect(options.webBuild).toBe(true)
    })

    it('supports the legacy --development-server spelling', () => {
        expect(resolveBuildOptions(['--development-server']).devServer).toBe('ON')
    })

    it('parses the parallel job count', () => {
        expect(resolveBuildOptions(['-j', '4']).parallel).toBe(4)
        expect(resolveBuildOptions(['-j=8']).parallel).toBe(8)
        expect(() => resolveBuildOptions(['-j', 'many'])).toThrow('Invalid parallel job count')
    })

    it('rejects invalid build types and unknown flags', () => {
        expect(() => resolveBuildOptions(['-b', 'Prod'])).toThrow('Invalid build type: Prod')
        expect(() => resolveBuildOptions(['--frobnicate'])).toThrow()
    })
})
