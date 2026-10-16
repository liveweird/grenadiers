import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [react()],
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
        'src/main.tsx',
        'src/**/*.d.ts',
      ],
      thresholds: { lines: 85, statements: 85, functions: 85, branches: 85 },
    },
  },
})
