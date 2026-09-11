/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck

import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
    // rollupTypes is off: api-extractor drops `declare global` blocks, which
    // would silently strip the Window.__BBX__ augmentation from init.d.ts.
    plugins: [tsconfigPaths(), dts({ insertTypesEntry: true, rollupTypes: false })],
    build: {
        target: 'esnext',
        // Not minified: the CLI runs in dev shells and CMake builds, where readable
        // stack traces matter more than bundle size (all deps are external anyway).
        minify: false,
        sourcemap: false,
        lib: {
            name: '@bbx-audio/plugin',
            formats: ['es'],
            entry: {
                index: resolve(__dirname, './src/index.ts'),
                init: resolve(__dirname, './src/init.ts'),
                presets: resolve(__dirname, './src/presets.ts'),
                params: resolve(__dirname, './src/params.ts'),
                vite: resolve(__dirname, './src/vite.ts'),
                cli: resolve(__dirname, './src/cli/main.ts'),
            },
        },
        rollupOptions: {
            external: [/^node:/, /^@bbx-audio\//, 'vite'],
        },
    },
})
