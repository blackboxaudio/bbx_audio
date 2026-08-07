import { copyFileSync, existsSync, mkdirSync, rmSync } from 'node:fs'
import { availableParallelism } from 'node:os'
import { join } from 'node:path'
import { parseArgs } from 'node:util'

import { HELP_COMMON_OPTIONS, normalizeArgs } from '../args'
import { parseEnvFile, requireEnv, resolvePluginContext } from '../context'
import { logCommand, run, setVerbose } from '../exec'
import {
    banner,
    boxEmpty,
    boxFooter,
    boxHeader,
    boxLine,
    die,
    formatDuration,
    getSize,
    progressInit,
    progressStep,
} from '../ui'

const HELP = `Usage: bbx build [OPTIONS]

Build the plugin (VST3 and AU).

Options:
    -b, --build-type <TYPE>
                         Build type: Release (default) or Debug
    -c, --copy           Copy built plugins to system plugin folders
    -d, --dev-server     Use dev server (localhost:5173) instead of bundled assets
    -r, --remove-prev-build
                         Remove previous build directory before building
    -s, --skip-signature Skip code signing (macOS only)
    -w, --web-target     Build web assets before plugin
    -j, --parallel <N>   Parallel build jobs (default: CPU count)
${HELP_COMMON_OPTIONS}`

export interface BuildOptions {
    buildType: 'Release' | 'Debug'
    logging: 'ON' | 'OFF'
    devServer: 'ON' | 'OFF'
    copy: boolean
    removePrev: boolean
    skipSign: boolean
    webBuild: boolean
    parallel: number
    verbose: boolean
    help: boolean
}

export function resolveBuildOptions(argv: string[]): BuildOptions {
    const { values } = parseArgs({
        args: normalizeArgs(argv),
        options: {
            'build-type': { type: 'string', short: 'b' },
            copy: { type: 'boolean', short: 'c' },
            'dev-server': { type: 'boolean', short: 'd' },
            'development-server': { type: 'boolean' },
            'remove-prev-build': { type: 'boolean', short: 'r' },
            'skip-signature': { type: 'boolean', short: 's' },
            'web-target': { type: 'boolean', short: 'w' },
            parallel: { type: 'string', short: 'j' },
            help: { type: 'boolean', short: 'h' },
            verbose: { type: 'boolean', short: 'v' },
        },
        strict: true,
        allowPositionals: false,
    })

    const buildType = values['build-type'] ?? 'Release'
    if (buildType !== 'Release' && buildType !== 'Debug') {
        throw new Error(`Invalid build type: ${buildType} (expected Release or Debug)`)
    }

    let parallel = availableParallelism()
    if (values.parallel !== undefined) {
        parallel = Number.parseInt(values.parallel, 10)
        if (!Number.isInteger(parallel) || parallel < 1) {
            throw new Error(`Invalid parallel job count: ${values.parallel}`)
        }
    }

    return {
        buildType,
        logging: buildType === 'Debug' ? 'ON' : 'OFF',
        devServer: values['dev-server'] || values['development-server'] ? 'ON' : 'OFF',
        copy: values.copy ?? false,
        removePrev: values['remove-prev-build'] ?? false,
        skipSign: values['skip-signature'] ?? false,
        webBuild: values['web-target'] ?? false,
        parallel,
        verbose: values.verbose ?? false,
        help: values.help ?? false,
    }
}

export function buildCmakeConfigureArgs(options: BuildOptions): string[] {
    return [
        '-S',
        '.',
        '-B',
        'bin',
        `-DCMAKE_BUILD_TYPE=${options.buildType}`,
        `-DBBX_USE_DEV_SERVER=${options.devServer}`,
        `-DBBX_ENABLE_LOGGING=${options.logging}`,
    ]
}

export function runBuild(argv: string[]): void {
    banner('build')

    let options: BuildOptions
    try {
        options = resolveBuildOptions(argv)
    } catch (error) {
        die(error instanceof Error ? error.message : String(error))
    }
    if (options.help) {
        console.log(HELP)
        return
    }
    setVerbose(options.verbose)

    const { root, env } = resolvePluginContext()
    process.chdir(root)
    const pluginName = requireEnv(env, 'PLUGIN_NAME')
    const companyName = requireEnv(env, 'COMPANY_NAME')

    const startTime = Date.now()
    const darwin = process.platform === 'darwin'
    const signing = darwin && !options.skipSign

    if (darwin && options.copy && !run('sudo', ['-v'])) {
        die('Must be administrator to copy files')
    }

    let totalSteps = 2
    if (options.removePrev) totalSteps++
    if (options.webBuild) totalSteps++
    if (signing) totalSteps += 2
    if (options.copy) totalSteps++
    progressInit(totalSteps)

    if (options.removePrev) {
        progressStep('Removing previous build directory')
        logCommand('rm', ['-rf', 'bin'])
        rmSync('bin', { recursive: true, force: true })
    }

    if (options.webBuild) {
        progressStep('Building web assets')
        if (!run('yarn', ['build'], { cwd: 'web' })) {
            die('Failed to build web assets')
        }
    }

    progressStep('Configuring CMake')
    if (!run('cmake', buildCmakeConfigureArgs(options))) {
        die('CMake configuration failed')
    }

    progressStep('Compiling binaries')
    if (!run('cmake', ['--build', 'bin', '--config', options.buildType, '--parallel', String(options.parallel)])) {
        die('Compilation failed')
    }

    const vst3Path = `bin/${pluginName}_artefacts/${options.buildType}/VST3/${pluginName}.vst3`
    const auPath = `bin/${pluginName}_artefacts/${options.buildType}/AU/${pluginName}.component`

    if (signing) {
        if (!existsSync('.env')) {
            die('No .env file present (required for signing)')
        }
        const identity = parseEnvFile('.env')['APPLE_DEVELOPER_ID_APP']
        if (!identity) {
            die('APPLE_DEVELOPER_ID_APP not set in .env (required for signing)')
        }

        const codesignArgs = ['--deep', '--strict', '--options=runtime', '--timestamp']
        progressStep('Signing VST3')
        if (!run('codesign', ['--force', '-s', identity, '-v', vst3Path, ...codesignArgs])) {
            die('Failed to sign VST3')
        }
        run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', vst3Path])

        progressStep('Signing AU')
        if (!run('codesign', ['--force', '-s', identity, '-v', auPath, ...codesignArgs])) {
            die('Failed to sign AU')
        }
        run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', auPath])
    }

    if (options.copy) {
        progressStep('Copying plugins to system folders')
        if (darwin) {
            const vst3Dir = `/Library/Audio/Plug-Ins/VST3/${companyName}`
            run('sudo', ['mkdir', '-p', vst3Dir])
            run('sudo', ['rm', '-rf', `${vst3Dir}/${pluginName}.vst3`])
            if (!run('sudo', ['cp', '-r', vst3Path, `${vst3Dir}/${pluginName}.vst3`])) {
                die('Failed to copy VST3')
            }

            const auDir = `/Library/Audio/Plug-Ins/Components/${companyName}`
            run('sudo', ['mkdir', '-p', auDir])
            run('sudo', ['rm', '-rf', `${auDir}/${pluginName}.component`])
            if (!run('sudo', ['cp', '-r', auPath, `${auDir}/${pluginName}.component`])) {
                die('Failed to copy AU')
            }
        } else {
            // Windows: the VST3 artefact bundle contains a single binary; copying it
            // to Common Files requires an elevated shell.
            const commonFiles = process.env.COMMONPROGRAMFILES ?? 'C:\\Program Files\\Common Files'
            const destDir = join(commonFiles, 'VST3', companyName)
            try {
                mkdirSync(destDir, { recursive: true })
                rmSync(join(destDir, `${pluginName}.vst3`), { force: true })
                copyFileSync(
                    join(vst3Path, 'Contents', 'x86_64-win', `${pluginName}.vst3`),
                    join(destDir, `${pluginName}.vst3`)
                )
            } catch {
                die('Failed to copy VST3 (run from an elevated shell?)')
            }
        }
    }

    const elapsed = Math.round((Date.now() - startTime) / 1000)
    console.log()
    boxHeader('Build Summary')
    boxLine('Plugin', `${pluginName} v${requireEnv(env, 'PLUGIN_VERSION')}`)
    boxLine('Build Type', options.buildType)
    boxLine('Dev Server', options.devServer)
    boxLine('Logging', options.logging)
    boxEmpty()
    boxLine('VST3 Output', vst3Path)
    boxLine('VST3 Size', getSize(vst3Path))
    if (darwin) {
        boxLine('AU Output', auPath)
        boxLine('AU Size', getSize(auPath))
    }
    boxEmpty()
    boxLine('Total Time', formatDuration(elapsed))
    boxFooter()
}
