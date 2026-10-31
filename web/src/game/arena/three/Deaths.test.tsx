import ReactThreeTestRenderer from '@react-three/test-renderer'
import { BufferGeometry, MeshBasicMaterial, Texture, type Mesh } from 'three'
import { describe, expect, it, vi } from 'vitest'
import type { PresentationDeathCue } from '../types'
import type { ThreeAtlas } from './contracts'
import { Deaths } from './Deaths'

function testAtlas(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  return {
    texture: new Texture(), sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({
      id, sourceRect: { x: 0, y: 0, width: 80, height: 80 },
      localBounds: { x: 0, y: 0, width: 80, height: 80 }, pivot: { x: 40, y: 40 },
    }),
    geometry(id, anchor = 'pivot') {
      const key = `${id}:${anchor}`
      const geometry = geometries.get(key) ?? new BufferGeometry()
      geometries.set(key, geometry)
      return geometry
    },
    dispose() {},
  }
}

const cue = (appearance: PresentationDeathCue['appearance']): PresentationDeathCue => ({
  key: `cue:${appearance}`, appearance, entityId: 1,
  position: { x: 1536, y: 2560 }, heading: 'south', tick: 2, index: 0, bornAt: 0,
})

const meshNamed = (
  renderer: Awaited<ReturnType<typeof ReactThreeTestRenderer.create>>, name: string,
) => renderer.scene.find(({ instance }) => instance.name === name).instance as Mesh

describe('mounted death billboards', () => {
  it('advances adopted frames, stays detached from live actor cues, and disposes materials', async () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const dispose = vi.spyOn(MeshBasicMaterial.prototype, 'dispose')
    const atlas = testAtlas()
    const renderer = await ReactThreeTestRenderer.create(
      <Deaths cues={[cue('one'), cue('mob')]} atlas={atlas} reducedMotion={false} />,
    )
    const actor = meshNamed(renderer, 'billboard:actor.one.death.0')
    const mob = meshNamed(renderer, 'billboard:mob.death.0')
    expect(actor.geometry).toBe(atlas.geometry('actor.one.death.0', 'feet'))
    expect(renderer.scene.find(({ instance }) => instance.name === 'death:cue:one').instance.position)
      .toMatchObject({ x: 1.5, z: 2.5 })
    expect(renderer.scene.findAll(({ instance }) => instance.name.startsWith('billboard:actor.cue.')))
      .toEqual([])
    now = 151
    await renderer.advanceFrames(1, 0.016)
    expect(actor.geometry).toBe(atlas.geometry('actor.one.death.1', 'feet'))
    expect(mob.geometry).toBe(atlas.geometry('mob.death.1', 'feet'))
    now = 451
    await renderer.advanceFrames(1, 0.016)
    expect(actor.geometry).toBe(atlas.geometry('actor.one.death.3', 'feet'))
    now = 600
    await renderer.advanceFrames(1, 0.016)
    expect(renderer.scene.find(({ instance }) => instance.name === 'death:cue:one').instance.visible).toBe(false)
    await renderer.unmount()
    expect(dispose).toHaveBeenCalledTimes(4)
    vi.restoreAllMocks()
  })

  it('uses frame three as a short static reduced-motion presentation', async () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const atlas = testAtlas()
    const renderer = await ReactThreeTestRenderer.create(
      <Deaths cues={[cue('four')]} atlas={atlas} reducedMotion />,
    )
    expect(meshNamed(renderer, 'billboard:actor.four.death.3').geometry)
      .toBe(atlas.geometry('actor.four.death.3', 'feet'))
    now = 180
    await renderer.advanceFrames(1, 0.016)
    expect(renderer.scene.find(({ instance }) => instance.name === 'death:cue:four').instance.visible)
      .toBe(false)
    await renderer.unmount()
    vi.restoreAllMocks()
  })
})
