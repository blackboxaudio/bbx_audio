import { spawnSync } from 'node:child_process'

import { dim } from './ui'

let verbose = false

export function setVerbose(value: boolean): void {
    verbose = value
}

export function logCommand(command: string, args: string[]): void {
    if (verbose) {
        console.log(dim(`   $ ${[command, ...args].join(' ')}`))
    }
}

// Windows resolves yarn/cmake shims through cmd.exe; args with spaces must then
// be quoted by hand because Node does not quote them when `shell` is set.
const useShell = process.platform === 'win32'

function shellArgs(args: string[]): string[] {
    if (!useShell) return args
    return args.map((arg) => (/\s/.test(arg) ? `"${arg}"` : arg))
}

/** Run a command with inherited stdio; returns true when it exits 0. */
export function run(command: string, args: string[], options: { cwd?: string } = {}): boolean {
    logCommand(command, args)
    const result = spawnSync(command, shellArgs(args), { stdio: 'inherit', cwd: options.cwd, shell: useShell })
    return result.status === 0
}

/** Run a command, suppressing its output unless verbose; dumps output on failure. */
export function runQuiet(command: string, args: string[], options: { cwd?: string } = {}): boolean {
    logCommand(command, args)
    if (verbose) {
        const result = spawnSync(command, shellArgs(args), { stdio: 'inherit', cwd: options.cwd, shell: useShell })
        return result.status === 0
    }
    const result = spawnSync(command, shellArgs(args), { encoding: 'utf8', cwd: options.cwd, shell: useShell })
    if (result.status !== 0) {
        if (result.stdout) process.stdout.write(result.stdout)
        if (result.stderr) process.stderr.write(result.stderr)
        return false
    }
    return true
}

/** Run a command and capture its stdout. */
export function capture(
    command: string,
    args: string[],
    options: { cwd?: string } = {}
): { ok: boolean; stdout: string } {
    logCommand(command, args)
    const result = spawnSync(command, shellArgs(args), { encoding: 'utf8', cwd: options.cwd, shell: useShell })
    return { ok: result.status === 0, stdout: result.stdout ?? '' }
}
