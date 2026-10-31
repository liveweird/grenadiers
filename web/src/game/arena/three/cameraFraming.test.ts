import { describe, expect, it } from 'vitest'
import { Vector3 } from 'three'
import { applyArenaCameraOffset, createArenaCamera, fitArenaCamera } from './camera'

function sceneryCorners(width: number, height: number): Vector3[] {
  const bounds = [
    { minX: -1, maxX: width + 1, minZ: -1, maxZ: height + 1, minY: -.12, maxY: .22 },
    { minX: 0, maxX: width, minZ: 0, maxZ: height, minY: 0, maxY: 1.1 },
  ]
  return bounds.flatMap((box) => {
    const points: Vector3[] = []
    for (const x of [box.minX, box.maxX]) {
      for (const z of [box.minZ, box.maxZ]) {
        for (const y of [box.minY, box.maxY]) points.push(new Vector3(x, y, z))
      }
    }
    return points
  })
}

describe('round camera framing envelope', () => {
  it.each([[11, 9], [13, 11], [15, 13]] as const)(
    'keeps the whole %sx%s tray and scenery visible throughout every effect', (width, height) => {
      const board = { width, height }
      for (const viewport of [{ width: 320, height: 640 }, { width: 600, height: 600 }, { width: 1200, height: 500 }]) {
        const camera = createArenaCamera()
        fitArenaCamera(camera, board, viewport)
        for (const offset of [
          { zoom: 1.075, yaw: 0 }, { zoom: .91, yaw: 0 },
          { zoom: 1, yaw: -7 * Math.PI / 180 }, { zoom: 1, yaw: 7 * Math.PI / 180 },
        ]) {
          applyArenaCameraOffset(camera, board, offset)
          // Includes the actual tray and a conservative 1.1-unit interior scenery bound.
          for (const corner of sceneryCorners(width, height)) {
            const projected = corner.clone().project(camera)
            expect(Math.abs(projected.x)).toBeLessThan(1)
            expect(Math.abs(projected.y)).toBeLessThan(1)
            expect(Math.abs(projected.z)).toBeLessThan(1)
          }
        }
      }
    },
  )
})
