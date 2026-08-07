import { parseArgs } from 'node:util'

import { HELP_COMMON_OPTIONS, normalizeArgs } from '../args'
import { requireEnv, resolvePluginContext } from '../context'
import { capture, run, setVerbose } from '../exec'
import { banner, die, header, progressInit, progressStep, success } from '../ui'

const HELP = `Usage: bbx tag [OPTIONS]

Create and push a git tag for the version in plugin.env (vPLUGIN_VERSION).

Options:
    -d, --delete         Delete the tag locally and from remote
${HELP_COMMON_OPTIONS}`

export function runTag(argv: string[]): void {
    banner('tag')

    let values: { delete?: boolean; help?: boolean; verbose?: boolean }
    try {
        values = parseArgs({
            args: normalizeArgs(argv),
            options: {
                delete: { type: 'boolean', short: 'd' },
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

    const { root, env } = resolvePluginContext()
    process.chdir(root)
    const tag = `v${requireEnv(env, 'PLUGIN_VERSION')}`

    const localTagExists = capture('git', ['tag', '-l', tag]).stdout.trim() !== ''

    if (values.delete) {
        header(`Deleting Tag ${tag}`)

        const remoteTagExists =
            capture('git', ['ls-remote', '--tags', 'origin', `refs/tags/${tag}`]).stdout.trim() !== ''
        const totalSteps = (localTagExists ? 1 : 0) + (remoteTagExists ? 1 : 0)
        progressInit(Math.max(totalSteps, 1))

        if (localTagExists) {
            progressStep('Removing local tag')
            if (!run('git', ['tag', '-d', tag])) {
                die(`Failed to delete local tag ${tag}`)
            }
        }

        if (remoteTagExists) {
            progressStep('Removing remote tag')
            if (!run('git', ['push', '-d', 'origin', tag])) {
                die(`Failed to delete remote tag ${tag}`)
            }
        }

        success(`Tag ${tag} deleted`)
    } else {
        header(`Creating Tag ${tag}`)

        progressInit(localTagExists ? 3 : 2)

        if (localTagExists) {
            progressStep('Removing existing local tag')
            if (!run('git', ['tag', '-d', tag])) {
                die(`Failed to delete existing local tag ${tag}`)
            }
        }

        progressStep('Creating tag')
        if (!run('git', ['tag', tag])) {
            die(`Failed to create tag ${tag}`)
        }

        progressStep('Pushing to origin')
        if (!run('git', ['push', '-u', 'origin', tag])) {
            die(`Failed to push tag ${tag} to origin`)
        }

        success(`Tag ${tag} pushed to origin`)
    }
}
