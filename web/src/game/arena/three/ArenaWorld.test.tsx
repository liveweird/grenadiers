import { create } from '@react-three/test-renderer'
import { describe, expect, it, vi } from 'vitest'
import {
  Box3, BufferAttribute, BufferGeometry, DirectionalLight, InstancedMesh, Matrix4, Texture, Vector3,
} from 'three'
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

function distinctGeometryAtlas(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  const geometry = (id: string) => {
    let result = geometries.get(id)
    if (!result) {
      result = createFrameGeometry({
        id,
        sourceRect: { x: 0, y: 0, width: 48, height: 48 },
        localBounds: { x: -24, y: -24, width: 48, height: 48 },
        pivot: { x: 24, y: 24 },
      }, { width: 48, height: 48 }, 'center')
      geometries.set(id, result)
    }
    return result
  }
  return {
    texture: new Texture(),
    sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({
      id,
      sourceRect: { x: 0, y: 0, width: 48, height: 48 },
      localBounds: { x: -24, y: -24, width: 48, height: 48 },
      pivot: { x: 24, y: 24 },
    }),
    geometry,
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

describe('ArenaWorld lair geometry', () => {
  it('mounts the lair artwork upright on the front face within the beveled body', async () => {
    const atlas = distinctGeometryAtlas()
    const snapshot = gameSnapshot()
    snapshot.state.lairs = [{
      id: 1, cell: { column: 3, row: 2 }, spawnRemainingTicks: 0, livingMobs: 0,
    }]
    snapshot.state.arena.terrain[2 * snapshot.state.arena.width + 3] = 'wall'
    const renderer = await create(<ArenaWorld snapshot={snapshot} atlas={atlas} />)
    const entranceGeometry = atlas.geometry('terrain.lair', 'center')
    const entrance = renderer.scene.findAll(({ instance }) => (
      instance instanceof InstancedMesh && instance.geometry === entranceGeometry
    ))[0]?.instance as InstancedMesh | undefined
    expect(entrance).toBeDefined()

    const matrix = new Matrix4()
    entrance?.getMatrixAt(0, matrix)
    const normal = new Vector3(0, 0, 1).transformDirection(matrix)
    expect(normal.toArray()).toEqual([0, 0, 1])

    const positions = entranceGeometry.getAttribute('position') as BufferAttribute
    const bounds = new Box3().setFromBufferAttribute(positions)
      .applyMatrix4(matrix)
    expect(bounds.min.x).toBeCloseTo(3.15)
    expect(bounds.max.x).toBeCloseTo(3.85)
    expect(bounds.min.y).toBeCloseTo(0.0275)
    expect(bounds.max.y).toBeCloseTo(0.7275)
    expect(bounds.min.z).toBeCloseTo(2.913)
    expect(bounds.max.z).toBeCloseTo(2.913)

    expect(bounds.min.x).toBeGreaterThan(3.5 - 0.82 / 2)
    expect(bounds.max.x).toBeLessThan(3.5 + 0.82 / 2)
    expect(bounds.min.y).toBeGreaterThan(0)
    expect(bounds.max.y).toBeLessThan(0.8 - 0.045)

    const wallGeometry = atlas.geometry('material.wall.top', 'center')
    const wallTops = renderer.scene.findAll(({ instance }) => (
      instance instanceof InstancedMesh && instance.geometry === wallGeometry
    ))[0]?.instance as InstancedMesh | undefined
    const { width, height } = snapshot.state.arena
    expect(wallTops?.count).toBe(width * 2 + (height - 2) * 2)
    const wallPosition = new Vector3()
    for (let index = 0; index < (wallTops?.count ?? 0); index += 1) {
      wallTops?.getMatrixAt(index, matrix)
      wallPosition.setFromMatrixPosition(matrix)
      expect([wallPosition.x, wallPosition.z]).not.toEqual([3.5, 2.5])
    }
    await renderer.unmount()
  })
})
