import { resolve } from 'node:path'
import { defineConfig } from 'vite'

export default defineConfig({
  root: resolve(import.meta.dirname, 'renderer-diagnostics'),
  base: '/',
  build: {
    outDir: resolve(import.meta.dirname, 'renderer-diagnostics-dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        lifecycle: resolve(import.meta.dirname, 'renderer-diagnostics/index.html'),
        terrainPixel: resolve(import.meta.dirname, 'renderer-diagnostics/terrain-pixel.html'),
        artAtlas: resolve(import.meta.dirname, 'renderer-diagnostics/art-atlas.html'),
        atlasGameplay: resolve(import.meta.dirname, 'renderer-diagnostics/atlas-gameplay.html'),
        threeArena: resolve(import.meta.dirname, 'renderer-diagnostics/three-arena.html'),
      },
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 5188,
    strictPort: true,
  },
})
