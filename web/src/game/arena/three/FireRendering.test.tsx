import { create } from '@react-three/test-renderer'
import { BufferGeometry, Group, Mesh, MeshBasicMaterial, PointLight, Texture } from 'three'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import { FireEffects } from './FireEffects'
import { FireIllumination } from './FireIllumination'
import { createFireLighting } from './fireLighting'
import { createArenaSceneStore } from './sceneStore'

function atlas(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  return {
    texture: new Texture(),
    sourceManifest: {} as ThreeAtlas['sourceManifest'], manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({ id, sourceRect: { x: 0, y: 0, width: 48, height: 48 },
      localBounds: { x: 0, y: 0, width: 48, height: 48 }, pivot: { x: 0, y: 0 } }),
    geometry(id) {
      if (!geometries.has(id)) geometries.set(id, new BufferGeometry())
      return geometries.get(id)!
    },
    dispose() {
      for (const geometry of geometries.values()) geometry.dispose()
      this.texture.dispose()
    },
  }
}

afterEach(() => vi.restoreAllMocks())

describe('fire rendering ownership and presentation', () => {
  it('reuses four unshadowed lights, makes reduced illumination static, and disposes the pool', async () => {
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const view = (animated: boolean) => <FireIllumination
      model={createFireLighting(snapshot)} animated={animated}>{null}</FireIllumination>
    const renderer = await create(view(true))
    const lights = renderer.scene.findAll(({ instance }) => instance instanceof PointLight)
      .map(({ instance }) => instance as PointLight)
    expect(lights).toHaveLength(4)
    await renderer.advanceFrames(1, 1 / 60)
    expect(lights.every((light) => light.intensity === 0 && !light.castShadow)).toBe(true)
    snapshot.state.flames = [{ cell: { column: 3, row: 3 }, expiresAt: 200 }]
    await renderer.update(view(true))
    await renderer.advanceFrames(1, 1 / 60)
    expect(lights[0].intensity).toBeGreaterThan(0)
    expect(renderer.scene.findAll(({ instance }) => instance instanceof PointLight)
      .map(({ instance }) => instance)).toEqual(lights)
    now = 150
    await renderer.advanceFrames(1, 1 / 60)
    const movingIntensity = lights[0].intensity
    await renderer.update(view(false))
    await renderer.advanceFrames(1, 1 / 60)
    expect(lights[0].intensity).not.toBe(movingIntensity)
    const staticIntensity = lights[0].intensity
    now = 550
    await renderer.advanceFrames(1, 1 / 60)
    expect(lights[0].intensity).toBe(staticIntensity)
    snapshot.state.flames = []
    await renderer.update(view(false))
    await renderer.advanceFrames(1, 1 / 60)
    expect(lights.every((light) => light.intensity === 0)).toBe(true)
    const dispose = lights.map((light) => vi.spyOn(light, 'dispose'))
    await renderer.unmount()
    expect(dispose.every((spy) => spy.mock.calls.length === 1)).toBe(true)
  })

  it('warms only opted-in sprite artwork at its displayed world position and restores the base tint', async () => {
    const snapshot = gameSnapshot()
    snapshot.state.flames = [{ cell: { column: 3, row: 3 }, expiresAt: 200 }]
    snapshot.state.oilBarrels = []; snapshot.state.arena.blocks = []
    const art = atlas()
    const view = () => <FireIllumination model={createFireLighting(snapshot)} animated={false}>
      <group position={[3.5, 0, 3.5]}>
        <Billboard atlas={art} frameId="actor" fireTint />
        <Billboard atlas={art} frameId="status" />
      </group>
    </FireIllumination>
    const renderer = await create(view())
    await renderer.advanceFrames(1, 1 / 60)
    const material = (id: string) => (renderer.scene.find(({ instance }) =>
      instance.name === `billboard:${id}`).instance as Mesh<BufferGeometry, MeshBasicMaterial>).material
    expect(material('actor').color.r).toBeGreaterThan(1)
    expect(material('actor').color.b).toBeLessThan(1)
    expect(material('status').color.toArray()).toEqual([1, 1, 1])
    snapshot.state.flames = []
    await renderer.update(view())
    await renderer.advanceFrames(1, 1 / 60)
    expect(material('actor').color.toArray()).toEqual([1, 1, 1])
    await renderer.unmount(); art.dispose()
  })

  it('cycles all six artist frames, caps particles and ignition, expires flashes without another snapshot', async () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const snapshot = gameSnapshot()
    snapshot.state.flames = []; snapshot.state.oilBarrels = []
    const store = createArenaSceneStore()
    store.update(snapshot, now)
    snapshot.state.physicalTick! += 3
    snapshot.state.flames = Array.from({ length: 20 }, (_, index) => ({
      cell: { column: index % 15, row: Math.floor(index / 15) + 3 }, expiresAt: 200,
    }))
    now = 50; store.update(snapshot, now)
    const art = atlas()
    const view = (reducedMotion = false) => <FireEffects fire={store.read().fire!}
      atlas={art} running reducedMotion={reducedMotion} />
    const renderer = await create(view())
    expect(renderer.scene.findAll(({ instance }) => instance.name.startsWith('ignition:'))).toHaveLength(8)
    expect(renderer.scene.findAll(({ instance }) => instance.name === 'fire-smoke').length).toBeLessThanOrEqual(4)
    expect(renderer.scene.findAll(({ instance }) => instance.name === 'fire-ember').length).toBeLessThanOrEqual(8)
    const flames = renderer.scene.findAll(({ instance }) => instance.name === 'billboard:effect.flame.0')
    for (const [index, pose] of [0, 2, 1, 3, 4, 5].entries()) {
      now = index * 80
      await renderer.advanceFrames(1, 1 / 60)
      expect((flames[0].instance as Mesh).geometry).toBe(art.geometry(`effect.flame.${pose}`))
    }
    expect(renderer.scene.findAll(({ instance }) => instance.name.startsWith('ignition:'))
      .every(({ instance }) => !instance.visible)).toBe(true)
    const smoke = renderer.scene.findAll(({ instance }) => instance.name === 'fire-smoke')[0].instance as Group
    expect(smoke.position.y).toBeGreaterThan(0.5)
    await renderer.update(view(true))
    await renderer.advanceFrames(1, 1 / 60)
    expect(renderer.scene.findAll(({ instance }) => instance.name.startsWith('ignition:'))).toHaveLength(0)
    expect(renderer.scene.findAll(({ instance }) => instance.name === 'fire-smoke')).toHaveLength(0)
    expect(renderer.scene.findAll(({ instance }) => instance.name === 'fire-ember')).toHaveLength(0)
    expect(renderer.scene.findAll(({ instance }) => instance.name === 'billboard:effect.flame.0')
      .every(({ instance }) => (instance as Mesh).geometry === art.geometry('effect.flame.0'))).toBe(true)
    await renderer.unmount(); art.dispose()
  })
})
