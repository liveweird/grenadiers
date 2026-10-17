import js from '@eslint/js'
import globals from 'globals'
import sonarjs from 'eslint-plugin-sonarjs'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['node_modules', 'playwright-report', 'test-results', 'playwright-pickup-report', 'pickup-test-results', '.playwright']),
  {
    files: ['**/*.{ts,mjs}'],
    extends: [js.configs.recommended, tseslint.configs.recommended, sonarjs.configs.recommended],
    languageOptions: { globals: globals.node },
    rules: {
      'sonarjs/no-clear-text-protocols': 'off',
      'sonarjs/parameterized-tests': 'off',
      'sonarjs/cognitive-complexity': ['error', 20],
      complexity: ['error', 20],
      'max-lines-per-function': ['error', { max: 160, skipBlankLines: true, skipComments: true }],
      'max-depth': ['error', 4],
      'max-params': ['error', 5],
    },
  },
])
