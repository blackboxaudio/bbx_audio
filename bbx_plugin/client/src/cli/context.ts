import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

import { findPluginRoot } from '../params/generate'
import { die } from './ui'

export type EnvVars = Record<string, string>

/** Parses a simple KEY=VALUE env file (plugin.env, .env), stripping quotes. */
export function parseEnvFile(path: string): EnvVars {
    const env: EnvVars = {}
    for (const line of readFileSync(path, 'utf8').split('\n')) {
        const match = line.match(/^([A-Za-z_][A-Za-z0-9_]*)=(.*)$/)
        if (!match) continue
        let value = match[2].trim()
        if (
            (value.startsWith('"') && value.endsWith('"') && value.length >= 2) ||
            (value.startsWith("'") && value.endsWith("'") && value.length >= 2)
        ) {
            value = value.slice(1, -1)
        }
        env[match[1]] = value
    }
    return env
}

export interface PluginContext {
    /** Root of the plugin repo (the directory containing plugin.env). */
    root: string
    /** Parsed plugin.env variables (PLUGIN_NAME, PLUGIN_VERSION, COMPANY_NAME, ...). */
    env: EnvVars
}

/**
 * Locates the plugin repo from the current working directory and loads its
 * plugin.env, or exits with a clear message when run outside a plugin repo.
 */
export function resolvePluginContext(): PluginContext {
    let root: string
    try {
        root = findPluginRoot(process.cwd())
    } catch (error) {
        die(error instanceof Error ? error.message : String(error))
    }
    return { root, env: parseEnvFile(join(root, 'plugin.env')) }
}

export function requireEnv(env: EnvVars, key: string): string {
    const value = env[key]
    if (!value) {
        die(`${key} not set in plugin.env`)
    }
    return value
}

export function requireLayout(root: string, relativePath: string): void {
    if (!existsSync(join(root, relativePath))) {
        die(`bbx expects the template-plugin layout: ${relativePath} not found under ${root}`)
    }
}
