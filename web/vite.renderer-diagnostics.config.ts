import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  root: resolve(import.meta.dirname, 'renderer-diagnostics'),
  base: '/',
  build: {
    outDir: resolve(import.meta.dirname, 'renderer-diagnostics-dist'),
    emptyOutDir: true,
  },
  preview: {
    host: '0.0.0.0',
    port: 5188,
    strictPort: true,
  },
})
