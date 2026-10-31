import { create } from '@react-three/test-renderer'
import { describe, expect, it, vi } from 'vitest'
import { DirectionalLight, InstancedMesh, Texture } from 'three'
import { gameSnapshot } from '../../../test/game-snapshot'
import { ArenaWorld } from './ArenaWorld'
import type { ThreeAtlas } from './contracts'
import { createFrameGeometry } from './frameGeometry'

function atlasFixture(): ThreeAtlas {
  const frame = {
    id: 'material.fixture',
    sourceRect: { x: 0, y: 0, width: 48, height: 48 },
    localBounds: { x: -24, y: -24, width: 48, height: 48 },
    pivot: { x: 24, y: 24 },
  }
  const geometry = createFrameGeometry(frame, { width: 48, height: 48 }, 'center')
  return {
    texture: new Texture(),
    sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: () => frame,
    geometry: () => geometry,
    dispose: vi.fn(),
  }
}

describe('ArenaWorld GPU ownership', () => {
  it('clears and remounts instance buffers while retaining atlas-owned geometry and texture', async () => {
    const atlas = atlasFixture()
    const geometry = atlas.geometry('material.fixture')
    const geometryDispose = vi.spyOn(geometry, 'dispose')
    const textureDispose = vi.spyOn(atlas.texture, 'dispose')
    const snapshot = gameSnapshot()
    const renderer = await create(<ArenaWorld snapshot={snapshot} atlas={atlas} />)
    const first = renderer.scene.findAll(({ instance }) => instance instanceof InstancedMesh)
      .map(({ instance }) => instance as InstancedMesh)
    const firstDisposals = first.map((mesh) => vi.spyOn(mesh, 'dispose'))
    expect(first.length).toBeGreaterThan(0)

    await renderer.update(null)
    firstDisposals.forEach((dispose) => expect(dispose).toHaveBeenCalledOnce())
    expect(geometryDispose).not.toHaveBeenCalled()
    expect(textureDispose).not.toHaveBeenCalled()

    await renderer.update(<ArenaWorld snapshot={snapshot} atlas={atlas} />)
    const second = renderer.scene.findAll(({ instance }) => instance instanceof InstancedMesh)
      .map(({ instance }) => instance as InstancedMesh)
    const secondDisposals = second.map((mesh) => vi.spyOn(mesh, 'dispose'))
    expect(second).toHaveLength(first.length)
    expect(second[0]).not.toBe(first[0])
    await renderer.unmount()
    secondDisposals.forEach((dispose) => expect(dispose).toHaveBeenCalledOnce())
    expect(geometryDispose).not.toHaveBeenCalled()
    expect(textureDispose).not.toHaveBeenCalled()
  })

  it('keeps the directional target in the rendered scene graph', async () => {
    const renderer = await create(<ArenaWorld snapshot={gameSnapshot()} atlas={atlasFixture()} />)
    const light = renderer.scene.findByType('DirectionalLight').instance as DirectionalLight
    expect(light.target.parent).not.toBeNull()
    expect(light.target.position.toArray()).toEqual([7.5, 0, 6.5])
    await renderer.unmount()
  })
})
