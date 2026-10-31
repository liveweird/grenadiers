import { create } from '@react-three/test-renderer'
import { describe, expect, it, vi } from 'vitest'
import { BufferGeometry, Mesh, MeshBasicMaterial, Texture } from 'three'
import type { ThreeAtlas } from './contracts'
import { SpikePits } from './SpikePits'

function atlasFixture(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  return {
    texture: new Texture(), sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({ id, sourceRect: { x: 0, y: 0, width: 48, height: 48 },
      localBounds: { x: 0, y: 0, width: 48, height: 48 }, pivot: { x: 0, y: 0 } }),
    geometry: (id) => {
      const geometry = geometries.get(id) ?? new BufferGeometry()
      geometries.set(id, geometry)
      return geometry
    },
    dispose: vi.fn(),
  }
}

describe('authoritative spike-pit presentation', () => {
  it('renders published phases on the floor and caps the presentation at four nodes', async () => {
    const atlas = atlasFixture()
    const pits = [
      { cell: { column: 3, row: 2 }, phase: 'idle' as const, remainingTicks: 0 },
      { cell: { column: 5, row: 2 }, phase: 'warning' as const, remainingTicks: 24 },
      { cell: { column: 3, row: 6 }, phase: 'raised' as const, remainingTicks: 90 },
      { cell: { column: 5, row: 6 }, phase: 'idle' as const, remainingTicks: 0 },
      { cell: { column: 7, row: 6 }, phase: 'raised' as const, remainingTicks: 1 },
    ]
    const renderer = await create(<SpikePits pits={pits} atlas={atlas} reducedMotion />)
    const meshes = renderer.scene.findAll(({ instance }) => instance instanceof Mesh)
    expect(meshes).toHaveLength(4)
    expect(meshes.map(({ instance }) => (instance as Mesh).geometry)).toEqual([
      atlas.geometry('terrain.spikePit.idle'), atlas.geometry('terrain.spikePit.warning'),
      atlas.geometry('terrain.spikePit.raised'), atlas.geometry('terrain.spikePit.idle'),
    ])
    expect(renderer.scene.findAllByProps({ name: 'spike-pits' })).toHaveLength(1)
    await renderer.unmount()
  })

  it('keeps reduced-motion warnings static and disposes only owned materials', async () => {
    const atlas = atlasFixture()
    const geometry = atlas.geometry('terrain.spikePit.warning')
    const geometryDispose = vi.spyOn(geometry, 'dispose')
    const renderer = await create(<SpikePits pits={[{
      cell: { column: 3, row: 3 }, phase: 'warning', remainingTicks: 12,
    }]} atlas={atlas} reducedMotion />)
    const mesh = renderer.scene.findAll(({ instance }) => instance instanceof Mesh)[0]!.instance as Mesh
    const materialDispose = vi.spyOn(mesh.material as MeshBasicMaterial, 'dispose')
    await renderer.unmount()
    expect(materialDispose).toHaveBeenCalledOnce()
    expect(geometryDispose).not.toHaveBeenCalled()
    expect(atlas.dispose).not.toHaveBeenCalled()
  })
})
