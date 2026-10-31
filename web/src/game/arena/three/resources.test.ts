import { describe, expect, it, vi } from 'vitest'
import { BufferGeometry, InstancedMesh, MeshStandardMaterial } from 'three'
import { replaceInstancedMesh } from './resources'

describe('world instance ownership', () => {
  it('releases instance buffers across clear and remount without disposing borrowed resources', () => {
    const geometry = new BufferGeometry()
    const material = new MeshStandardMaterial()
    const geometryDispose = vi.spyOn(geometry, 'dispose')
    const materialDispose = vi.spyOn(material, 'dispose')
    const first = new InstancedMesh(geometry, material, 3)
    const second = new InstancedMesh(geometry, material, 5)
    const firstDispose = vi.spyOn(first, 'dispose')
    const secondDispose = vi.spyOn(second, 'dispose')

    let current: InstancedMesh | null = null
    current = replaceInstancedMesh(current, first)
    current = replaceInstancedMesh(current, null)
    current = replaceInstancedMesh(current, second)
    current = replaceInstancedMesh(current, null)

    expect(current).toBeNull()
    expect(firstDispose).toHaveBeenCalledOnce()
    expect(secondDispose).toHaveBeenCalledOnce()
    expect(geometryDispose).not.toHaveBeenCalled()
    expect(materialDispose).not.toHaveBeenCalled()
  })
})
