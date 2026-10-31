import { describe, expect, it } from 'vitest'
import { assertProductionArenaModules } from './renderer-boundary'

const engine = [
  '/repo/web/node_modules/three/build/three.module.js',
  '/repo/web/node_modules/@react-three/fiber/dist/events.esm.js',
]

describe('production renderer module boundary', () => {
  it('accepts the selected engine with pure atlas metadata', () => {
    expect(() => assertProductionArenaModules([
      ...engine, '/repo/web/src/game/arena/artAtlas/descriptor.ts',
    ])).not.toThrow()
  })

  it.each([
    '/repo/web/node_modules/pixi.js/lib/index.mjs',
    '/repo/web/node_modules/@pixi/colord/lib/index.js',
    '/repo/web/src/game/arena/artAtlas/catalog.ts',
    'C:\\repo\\web\\src\\game\\arena\\pixiRenderer.ts',
  ])('rejects reference drawing code in emitted modules: %s', (moduleId) => {
    expect(() => assertProductionArenaModules([...engine, moduleId])).toThrow('Reference renderer')
  })

  it('rejects a build which accidentally drops either selected engine layer', () => {
    expect(() => assertProductionArenaModules(engine.slice(0, 1))).toThrow('Three.js and React Three Fiber')
    expect(() => assertProductionArenaModules(engine.slice(1))).toThrow('Three.js and React Three Fiber')
  })
})
