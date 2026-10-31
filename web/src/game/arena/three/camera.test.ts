import { describe, expect, it } from 'vitest'
import { Vector3 } from 'three'
import { createArenaCamera, fitArenaCamera } from './camera'
import { cellToWorld, simulationToWorld, WORLD_TILE } from './coordinates'

describe('Three arena coordinates and camera', () => {
  it('maps 1024 simulation units to one uncentered world tile', () => {
    expect(WORLD_TILE).toBe(1)
    expect(cellToWorld(4, 2)).toEqual([4.5, 2.5])
    expect(simulationToWorld(4608, 2560)).toEqual([4.5, 2.5])
  })

  it('centers supported boards in the fixed orthographic view', () => {
    const camera = createArenaCamera()
    fitArenaCamera(camera, { width: 11, height: 9 }, { width: 320, height: 640 })
    const direction = camera.getWorldDirection(new Vector3())
    const towardCenter = new Vector3(5.5, 0, 4.5).sub(camera.position).normalize()
    expect(direction.distanceTo(towardCenter)).toBeLessThan(0.000001)
    expect(camera.right - camera.left).toBeCloseTo((camera.top - camera.bottom) * 0.5)
    expect(camera.top - camera.bottom).toBeGreaterThan(11)

    fitArenaCamera(camera, { width: 15, height: 13 }, { width: 1200, height: 500 })
    expect(camera.right - camera.left).toBeGreaterThan(17)
    expect(camera.position.x).toBe(7.5)
  })
})
