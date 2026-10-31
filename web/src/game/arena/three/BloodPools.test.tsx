import ReactThreeTestRenderer from '@react-three/test-renderer'
import { BufferGeometry, MeshBasicMaterial, Texture, type Group, type Mesh } from 'three'
import { describe, expect, it, vi } from 'vitest'
import { BloodPools } from './BloodPools'
import type { BloodPool } from './bloodPoolState'
import type { ThreeAtlas } from './contracts'

function testAtlas(): ThreeAtlas {
  const geometry = new BufferGeometry()
  return {
    texture: new Texture(), sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({
      id, sourceRect: { x: 0, y: 0, width: 48, height: 48 },
      localBounds: { x: 0, y: 0, width: 48, height: 48 }, pivot: { x: 0, y: 0 },
    }),
    geometry: () => geometry,
    dispose() {},
  }
}

describe('mounted blood pools', () => {
  it('uses borrowed centered geometry as a depth-tested ground decal without a shadow', async () => {
    const dispose = vi.spyOn(MeshBasicMaterial.prototype, 'dispose')
    const disposeGeometry = vi.spyOn(BufferGeometry.prototype, 'dispose')
    const atlas = testAtlas()
    const pool: BloodPool = {
      key: 'blood:1:2', column: 1, row: 2, position: { x: 1536, y: 2560 },
    }
    const renderer = await ReactThreeTestRenderer.create(<BloodPools pools={[pool]} atlas={atlas} />)
    const root = renderer.scene.find(({ instance }) => instance.name === pool.key).instance as Group
    const mesh = renderer.scene.find(
      ({ instance }) => instance.name === 'billboard:effect.bloodPool',
    ).instance as Mesh
    expect(root.position).toMatchObject({ x: 1.5, y: 0.005, z: 2.5 })
    expect(mesh.geometry).toBe(atlas.geometry('effect.bloodPool', 'center'))
    expect(mesh.rotation.x).toBeCloseTo(-Math.PI / 2)
    expect(mesh.scale.x).toBeCloseTo(0.65 / 48)
    expect(mesh.renderOrder).toBe(1)
    expect((mesh.material as MeshBasicMaterial).depthTest).toBe(true)
    expect(renderer.scene.findAll(({ instance }) => instance.name.includes('shadow'))).toEqual([])
    await renderer.unmount()
    expect(dispose).toHaveBeenCalledOnce()
    expect(disposeGeometry).not.toHaveBeenCalled()
    vi.restoreAllMocks()
  })
})
