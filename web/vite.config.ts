import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'
import { productionArenaBoundary } from './scripts/renderer-boundary'

export default defineConfig({
  plugins: [react(), productionArenaBoundary()],
  server: {
    host: '127.0.0.1',
    port: 5177,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8085',
        ws: true,
      },
    },
  },
  test: {
    globals: true,
    environment: 'happy-dom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/api/schema.ts',
        'src/game/schema.ts',
        'src/realtime/schema.ts',
        'src/**/*.test.{ts,tsx}',
        'src/test/**',
        // Test-only fixtures are executed but do not belong in application coverage totals.
        'src/**/testSupport.ts',
        'src/main.tsx',
        'src/**/*.d.ts',
      ],
      thresholds: { lines: 90, statements: 88, functions: 88, branches: 88 },
    },
  },
})
