import { readdirSync } from 'node:fs'
import { join } from 'node:path'
import { parseArgs } from 'node:util'

import { HELP_COMMON_OPTIONS, normalizeArgs } from '../args'
import { resolvePluginContext } from '../context'
import { run, setVerbose } from '../exec'
import { banner, die, progressInit, progressStep, success, warn } from '../ui'

const HELP = `Usage: bbx format [OPTIONS]

Format C++ source code using clang-format and Rust code using rustfmt.

Formats files in:
    - src/
    - lib/cortex/src/
    - lib/dsp/src/
${HELP_COMMON_OPTIONS}`

function collectCppFiles(dir: string): string[] {
    const files: string[] = []
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = join(dir, entry.name)
        if (entry.isDirectory()) {
            files.push(...collectCppFiles(path))
        } else if (/\.(h|cpp)$/i.test(entry.name)) {
            files.push(path)
        }
    }
    return files
}

export function runFormat(argv: string[]): void {
    banner('format')

    let values: { help?: boolean; verbose?: boolean }
    try {
        values = parseArgs({
            args: normalizeArgs(argv),
            options: {
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

    progressInit(3)

    const formatDirectory = (dir: string, label: string): void => {
        progressStep(`Formatting ${label}`)
        let files: string[]
        try {
            files = collectCppFiles(dir)
        } catch {
            die(`Failed to format ${label} (${dir} not found)`)
        }
        if (files.length === 0) {
            warn(`No C++ sources found in ${dir}`)
            return
        }
        if (!run('clang-format', ['-i', '-style=file', ...files])) {
            die(`Failed to format ${label}`)
        }
    }

    formatDirectory('src', 'plugin source')
    formatDirectory(join('lib', 'cortex', 'src'), 'cortex library')

    progressStep('Formatting dsp library')
    if (!run('cargo', ['fmt', '--manifest-path', 'lib/dsp/Cargo.toml'])) {
        die('Failed to format dsp library')
    }

    success('All files formatted')
}
