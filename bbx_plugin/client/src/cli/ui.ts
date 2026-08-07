/**
 * Terminal output helpers for the bbx CLI — a TypeScript port of the
 * template's former `scripts/utils.sh` (colors, banners, progress, boxes),
 * kept dependency-free by hand-rolling the ANSI codes.
 */

import { existsSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const useColor = Boolean(
    process.stdout.isTTY && process.env.TERM && process.env.TERM !== 'dumb' && !process.env.NO_COLOR
)

const style =
    (code: string) =>
    (text: string): string =>
        useColor ? `\x1b[${code}m${text}\x1b[0m` : text

export const red = style('0;31')
export const green = style('0;32')
export const yellow = style('0;33')
export const cyan = style('0;36')
export const dim = style('2')
export const bold = style('1')
export const boldBlue = style('1;34')
export const boldGreen = style('1;32')

export function header(title: string): void {
    const line = '━'.repeat(50)
    console.log(`\n${boldBlue(line)}`)
    console.log(boldBlue(`  ${title}`))
    console.log(boldBlue(line))
}

export function step(message: string): void {
    console.log(`${cyan('▸')} ${message}`)
}

export function success(message: string): void {
    console.log(`${green('✓')} ${message}`)
}

export function warn(message: string): void {
    console.log(`${yellow('⚠')} ${message}`)
}

export function die(message: string): never {
    console.error(`\n${red(`✗ ${message}`)}`)
    process.exit(1)
}

const BANNERS = {
    build: String.raw`
 ____  _   _ ___ _     ____
| __ )| | | |_ _| |   |  _ \
|  _ \| | | || || |   | | | |
| |_) | |_| || || |___| |_| |
|____/ \___/|___|_____|____/`,
    format: String.raw`
 _____ ___  ____  __  __    _  _____
|  ___/ _ \|  _ \|  \/  |  / \|_   _|
| |_ | | | | |_) | |\/| | / _ \ | |
|  _|| |_| |  _ <| |  | |/ ___ \| |
|_|   \___/|_| \_\_|  |_/_/   \_\_|`,
    test: String.raw`
 _____ _____ ____ _____
|_   _| ____/ ___|_   _|
  | | |  _| \___ \ | |
  | | | |___ ___) || |
  |_| |_____|____/ |_|`,
    tag: String.raw`
 _____  _    ____
|_   _|/ \  / ___|
  | | / _ \| |  _
  | |/ ___ \ |_| |
  |_/_/   \_\____|`,
} as const

export function banner(name: keyof typeof BANNERS): void {
    console.log(boldBlue(BANNERS[name]))
    console.log()
}

let stepCurrent = 0
let stepTotal = 0

export function progressInit(total: number): void {
    stepCurrent = 0
    stepTotal = total
}

export function progressStep(message: string): void {
    stepCurrent++
    console.log(`${cyan(`[${stepCurrent}/${stepTotal}]`)} ${message}`)
}

const BOX_WIDTH = 72
const BOX_LABEL_WIDTH = 18

export function boxHeader(title: string): void {
    const innerWidth = BOX_WIDTH - 2
    const padding = Math.floor((innerWidth - title.length - 2) / 2)
    const padRight = innerWidth - title.length - 2 - padding
    console.log(boldGreen(`╭${'─'.repeat(innerWidth)}╮`))
    console.log(`${boldGreen('│')}${' '.repeat(padding)} ${bold(title)} ${' '.repeat(padRight)}${boldGreen('│')}`)
    console.log(boldGreen(`├${'─'.repeat(innerWidth)}┤`))
}

export function boxLine(label: string, value: string): void {
    const valueWidth = BOX_WIDTH - 2 - BOX_LABEL_WIDTH - 4
    console.log(
        `${boldGreen('│')}  ${dim(label.padEnd(BOX_LABEL_WIDTH))}${value.padEnd(valueWidth)}  ${boldGreen('│')}`
    )
}

export function boxEmpty(): void {
    console.log(`${boldGreen('│')}${' '.repeat(BOX_WIDTH - 2)}${boldGreen('│')}`)
}

export function boxFooter(): void {
    console.log(boldGreen(`╰${'─'.repeat(BOX_WIDTH - 2)}╯`))
}

export function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}m ${String(secs).padStart(2, '0')}s`
}

export function printElapsed(startTimeMs: number): void {
    const elapsed = Math.round((Date.now() - startTimeMs) / 1000)
    console.log(`\n${dim(`Completed in ${formatDuration(elapsed)}`)}`)
}

function sizeOf(path: string): number {
    const stats = statSync(path, { throwIfNoEntry: false })
    if (!stats) return 0
    if (stats.isDirectory()) {
        return readdirSync(path).reduce((total, entry) => total + sizeOf(join(path, entry)), 0)
    }
    return stats.size
}

/** Human-readable file or directory size, in the spirit of `du -sh`. */
export function getSize(path: string): string {
    if (!existsSync(path)) return 'N/A'
    const bytes = sizeOf(path)
    const units = ['B', 'K', 'M', 'G', 'T']
    let value = bytes
    let unit = 0
    while (value >= 1024 && unit < units.length - 1) {
        value /= 1024
        unit++
    }
    const rendered = unit === 0 ? String(value) : value < 10 ? value.toFixed(1) : String(Math.round(value))
    return `${rendered}${units[unit]}`
}
