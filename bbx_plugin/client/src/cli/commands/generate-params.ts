import { parseArgs } from 'node:util'

import { findPluginRoot, generateParams } from '../../params/generate'
import { HELP_COMMON_OPTIONS, normalizeArgs } from '../args'
import { setVerbose } from '../exec'
import { die } from '../ui'

const HELP = `Usage: bbx generate-params [--cpp <output-dir>] [--web]

Generate parameter ID definitions from the plugin's parameters.*.json files.

Options:
    --cpp <output-dir>   Emit bbx_generated_params.h into the given directory
    --web                Emit web/src/lib/generated/params.ts
${HELP_COMMON_OPTIONS}`

// No banner here: CMake invokes this on every build, so the output stays terse.
export function runGenerateParams(argv: string[]): void {
    let values: { cpp?: string; web?: boolean; help?: boolean; verbose?: boolean }
    try {
        values = parseArgs({
            args: normalizeArgs(argv),
            options: {
                cpp: { type: 'string' },
                web: { type: 'boolean' },
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

    if (!values.cpp && !values.web) {
        console.error('Usage: bbx generate-params [--cpp <output-dir>] [--web]')
        process.exit(1)
    }

    try {
        const repoRoot = findPluginRoot(process.cwd())
        const pluginType = generateParams({ repoRoot, cppOutDir: values.cpp ?? null, web: values.web ?? false })
        console.log(`Generated ${pluginType} parameter definitions`)
    } catch (error) {
        die(error instanceof Error ? error.message : String(error))
    }
}
