import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import sonarjs from 'eslint-plugin-sonarjs'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores([
    'dist',
    'coverage',
    'src/api/schema.ts',
    'src/realtime/schema.ts',
    'src/game/schema.ts',
    'src/game/v3/schema.ts',
  ]),
  {
    files: ['**/*.{ts,tsx,mjs}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite,
      sonarjs.configs.recommended,
    ],
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
    rules: {
      'sonarjs/no-clear-text-protocols': 'off',
      'sonarjs/no-hardcoded-passwords': 'off',
      'sonarjs/parameterized-tests': 'off',
      'sonarjs/no-nested-conditional': 'off',
      'sonarjs/cognitive-complexity': ['error', 20],
      complexity: ['error', 20],
      'max-lines-per-function': ['error', { max: 180, skipBlankLines: true, skipComments: true }],
      'max-depth': ['error', 4],
      'max-params': ['error', 5],
    },
  },
])
