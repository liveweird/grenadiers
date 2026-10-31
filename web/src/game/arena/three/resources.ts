import { InstancedMesh, MeshStandardMaterial } from 'three'
import type { ThreeAtlas } from './contracts'

export interface WorldResources {
  material: MeshStandardMaterial
  dispose: () => void
}

/** Resources are scoped to one Fiber root; borrowed atlas geometry remains atlas-owned. */
export function createWorldResources(atlas: ThreeAtlas): WorldResources {
  let material: MeshStandardMaterial | null = new MeshStandardMaterial({
    color: 0xffffff,
    map: atlas.texture,
    alphaTest: 0.05,
    roughness: 0.88,
    metalness: 0,
  })
  material.name = 'arena-world-material'
  return {
    get material() {
      if (!material) throw new Error('Arena world resources have been disposed')
      return material
    },
    dispose() {
      material?.dispose()
      material = null
    },
  }
}

/** Releases per-draw instance buffers without touching geometry or material borrowers. */
export function replaceInstancedMesh(
  previous: InstancedMesh | null, next: InstancedMesh | null,
): InstancedMesh | null {
  if (previous && previous !== next) previous.dispose()
  return next
}
