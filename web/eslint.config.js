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
    'src/realtime/v1/schema.ts',
    'src/game/schema.ts',
    'src/game/v3/schema.ts',
    'src/game/v4/schema.ts',
    'src/game/v5/schema.ts',
    'src/game/v6/schema.ts',
    'src/game/v7/schema.ts',
    'src/game/v8/schema.ts',
    'src/game/v9/schema.ts',
    'src/game/v10/schema.ts',
    'src/game/v11/schema.ts',
    'src/game/v12/schema.ts',
    'src/game/v13/schema.ts',
    'src/game/v14/schema.ts',
    'src/game/v15/schema.ts',
    'src/validation/generated/*.mjs',
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
      // Loopback HTTP/WS addresses are intentional development and contract fixtures.
      'sonarjs/no-clear-text-protocols': 'off',
      // Fixtures use explicit fake credentials; runtime capabilities come from the server.
      'sonarjs/no-hardcoded-passwords': 'off',
      // Explicit version/scenario assertions are valid; table-driven style is optional.
      'sonarjs/parameterized-tests': 'off',
      // Small typed projections may use ternaries; complexity and depth remain bounded.
      'sonarjs/no-nested-conditional': 'off',
      'sonarjs/cognitive-complexity': ['error', 20],
      complexity: ['error', 20],
      'max-lines-per-function': ['error', { max: 180, skipBlankLines: true, skipComments: true }],
      'max-depth': ['error', 4],
      'max-params': ['error', 5],
    },
  },
])
