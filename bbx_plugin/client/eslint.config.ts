import js from '@eslint/js'
import globals from 'globals'
import tseslint from 'typescript-eslint'
import { defineConfig } from 'eslint/config'

export default defineConfig([
    {
        ignores: ['dist', 'node_modules'],
    },
    {
        files: ['src/**/*.{js,mjs,cjs,ts,mts,cts}', 'test/**/*.ts', 'bin/*.mjs'],
        plugins: { js },
        extends: ['js/recommended'],
        languageOptions: { globals: { ...globals.browser, ...globals.node } },
    },
    tseslint.configs.recommended,
])
