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

  it.each([
    [11, 9, 320, 640], [11, 9, 1200, 500],
    [13, 11, 320, 640], [13, 11, 1200, 500],
    [15, 13, 320, 640], [15, 13, 1200, 500],
  ] as const)('resets zoom and retains the padded rim for %sx%s at %sx%s', (
    width, height, viewportWidth, viewportHeight,
  ) => {
    const camera = createArenaCamera()
    camera.zoom = 2
    fitArenaCamera(camera, { width, height }, { width: viewportWidth, height: viewportHeight })
    const usableWidth = (camera.right - camera.left) / 1.075
    const usableHeight = (camera.top - camera.bottom) / 1.075
    const rimWidth = width + 2.2
    const rimProjectedHeight = (height + 2.2) * Math.sin(55 * Math.PI / 180)
      + 1.1 * Math.cos(55 * Math.PI / 180)
    expect(camera.zoom).toBe(1)
    expect(usableWidth).toBeGreaterThanOrEqual(rimWidth)
    expect(usableHeight).toBeGreaterThanOrEqual(rimProjectedHeight)
  })
})
