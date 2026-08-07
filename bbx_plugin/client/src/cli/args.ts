/**
 * Normalizes legacy shell-script flag spellings before `util.parseArgs`:
 * the old scripts accepted `-b=Debug`, which parseArgs does not, so the
 * short-with-equals form is split into `-b Debug`.
 */
export function normalizeArgs(args: string[]): string[] {
    const normalized: string[] = []
    for (const arg of args) {
        const shortWithEquals = arg.match(/^(-[A-Za-z])=(.*)$/)
        if (shortWithEquals) {
            normalized.push(shortWithEquals[1], shortWithEquals[2])
        } else {
            normalized.push(arg)
        }
    }
    return normalized
}

export const HELP_COMMON_OPTIONS = `
Common Options:
    -h, --help           Show this help message
    -v, --verbose        Show commands as they execute`
