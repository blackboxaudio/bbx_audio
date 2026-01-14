/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck

import { resolve } from 'path'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'
import tsconfigPaths from 'vite-tsconfig-paths'

export default defineConfig({
    plugins: [tsconfigPaths(), dts({ insertTypesEntry: true, rollupTypes: true })],
    build: {
        target: 'esnext',
        minify: true,
        sourcemap: false,
        lib: {
            name: '@bbx-audio/net',
            formats: ['es'],
            entry: {
                index: resolve(__dirname, './src/index.ts'),
            },
        },
    },
})
