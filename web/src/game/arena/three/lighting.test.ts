import { describe, expect, it } from 'vitest'
import { DirectionalLight, Object3D, Scene, Vector3 } from 'three'
import { aimArenaLight, arenaLightDirection, arenaLightPlacement } from './lighting'

describe('arena directional light target', () => {
  it.each([[11, 9], [13, 11], [15, 13]])('aims through the %ix%i board center', (width, height) => {
    const scene = new Scene()
    const light = new DirectionalLight()
    const target = new Object3D()
    scene.add(light, target)
    aimArenaLight(light, target, width, height)
    scene.updateMatrixWorld(true)

    const placement = arenaLightPlacement(width, height)
    expect(new Vector3().setFromMatrixPosition(target.matrixWorld).toArray()).toEqual(placement.target)
    const expected = new Vector3(...placement.target).sub(new Vector3(...placement.position)).normalize()
    expect(arenaLightDirection(light).distanceTo(expected)).toBeLessThan(0.000001)
    expect(light.target).toBe(target)
    expect(target.parent).toBe(scene)
  })
})
