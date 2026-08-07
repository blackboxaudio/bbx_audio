import { existsSync } from 'node:fs'
import { parseArgs } from 'node:util'

import { HELP_COMMON_OPTIONS, normalizeArgs } from '../args'
import { resolvePluginContext } from '../context'
import { run, setVerbose } from '../exec'
import { banner, die, printElapsed, progressInit, progressStep, success, warn } from '../ui'

const HELP = `Usage: bbx test [OPTIONS]

Run the automated test suites.

Runs:
    - cargo test in lib/dsp (effect and synth configurations)
    - svelte-check + tsc in web/

Options:
    -r, --rust-only      Run only the Rust tests
    -w, --web-only       Run only the web checks
${HELP_COMMON_OPTIONS}`

export function runTest(argv: string[]): void {
    banner('test')

    let values: { 'rust-only'?: boolean; 'web-only'?: boolean; help?: boolean; verbose?: boolean }
    try {
        values = parseArgs({
            args: normalizeArgs(argv),
            options: {
                'rust-only': { type: 'boolean', short: 'r' },
                'web-only': { type: 'boolean', short: 'w' },
                help: { type: 'boolean', short: 'h' },
                verbose: { type: 'boolean', short: 'v' },
            },
            strict: true,
            allowPositionals: false,
        }).values
    } catch (error) {
        die(error instanceof Error ? error.message : String(error))
    }
    if (values.help) {
        console.log(HELP)
        return
    }
    setVerbose(values.verbose ?? false)

    const { root } = resolvePluginContext()
    process.chdir(root)
    const startTime = Date.now()

    progressInit(3)

    if (!values['web-only']) {
        progressStep('Testing dsp library (effect)')
        if (!run('cargo', ['test', '--manifest-path', 'lib/dsp/Cargo.toml'])) {
            die('Rust tests failed (effect)')
        }

        progressStep('Testing dsp library (synth)')
        if (!run('cargo', ['test', '--manifest-path', 'lib/dsp/Cargo.toml', '--features', 'synth'])) {
            die('Rust tests failed (synth)')
        }
    } else {
        progressStep('Skipping Rust tests')
        progressStep('Skipping Rust tests')
    }

    if (!values['rust-only']) {
        progressStep('Checking web code')
        if (!existsSync('web/node_modules')) {
            warn("web/node_modules missing; run 'yarn install' first. Skipping web checks.")
            // NOTE: 'yarn run check' because 'yarn check' invokes yarn v1's built-in integrity command
        } else if (!run('yarn', ['run', 'check'], { cwd: 'web' })) {
            die('Web checks failed')
        }
    } else {
        progressStep('Skipping web checks')
    }

    success('All tests passed')
    printElapsed(startTime)
}
