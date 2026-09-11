/**
 * Entry point for the `bbx` CLI. Executed at module load so both the published
 * `bin/bbx.mjs` shim and a direct `node dist/cli.js` (how CMake invokes the
 * codegen) behave identically. Deliberately absent from the package's exports
 * map — this module is a program, not a library.
 */

import { runBuild } from './commands/build'
import { runFormat } from './commands/format'
import { runGenerateParams } from './commands/generate-params'
import { runTag } from './commands/tag'
import { runTest } from './commands/test'
import { die } from './ui'

interface CliCommand {
    description: string
    run: (argv: string[]) => void
}

const commands: Record<string, CliCommand> = {
    build: { description: 'Build the plugin (VST3 and AU)', run: runBuild },
    test: { description: 'Run the Rust and web test suites', run: runTest },
    format: { description: 'Format C++ (clang-format) and Rust (rustfmt) sources', run: runFormat },
    tag: { description: 'Create or delete the git tag for the plugin.env version', run: runTag },
    'generate-params': { description: 'Generate parameter IDs from parameters.*.json', run: runGenerateParams },
}

function printUsage(): void {
    const lines = Object.entries(commands).map(([name, command]) => `    ${name.padEnd(18)}${command.description}`)
    console.log(`Usage: bbx <command> [options]

Shared tooling for plugins derived from blackboxaudio/template-plugin.

Commands:
${lines.join('\n')}

Run 'bbx <command> --help' for command-specific options.`)
}

const [commandName, ...rest] = process.argv.slice(2)

if (!commandName || commandName === '-h' || commandName === '--help') {
    printUsage()
    process.exit(commandName ? 0 : 1)
}

const command = commands[commandName]
if (!command) {
    printUsage()
    die(`Unknown command: ${commandName}`)
}

try {
    command.run(rest)
} catch (error) {
    die(error instanceof Error ? error.message : String(error))
}
