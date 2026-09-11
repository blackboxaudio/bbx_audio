import { join } from 'node:path'

import type { Plugin, ViteDevServer } from 'vite'

import { findPluginRoot, generateParams } from './params/generate'

export interface BbxParamsOptions {
    /** Plugin repo root; defaults to walking up from the current working directory. */
    repoRoot?: string
}

/**
 * Regenerates the web `ParameterId` module (`web/src/lib/generated/params.ts`)
 * from the plugin's `parameters.*.json` on config load, and again whenever the
 * parameter files or `plugin.env` change while the dev server is running.
 */
export function bbxParams(options: BbxParamsOptions = {}): Plugin {
    let repoRoot: string
    return {
        name: 'bbx:params',
        config() {
            repoRoot = options.repoRoot ?? findPluginRoot(process.cwd())
            generateParams({ repoRoot, web: true })
        },
        configureServer(server: ViteDevServer) {
            const watched = ['plugin.env', 'parameters.effect.json', 'parameters.synth.json'].map((file) =>
                join(repoRoot, file)
            )
            for (const file of watched) {
                server.watcher.add(file)
            }
            server.watcher.on('change', (file) => {
                if (watched.includes(file)) {
                    generateParams({ repoRoot, web: true })
                }
            })
        },
    }
}
