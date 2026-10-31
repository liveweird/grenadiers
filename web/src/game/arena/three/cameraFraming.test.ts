import { describe, expect, it } from 'vitest'
import { Vector3 } from 'three'
import { applyArenaCameraOffset, createArenaCamera, fitArenaCamera } from './camera'

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
          // The actual tray extends one tile past the board, from -0.12 to 0.22 in height.
          // Interior scenery is conservatively bounded at 1.1 world units.
          for (const bounds of [
            { minX: -1, maxX: width + 1, minZ: -1, maxZ: height + 1, minY: -.12, maxY: .22 },
            { minX: 0, maxX: width, minZ: 0, maxZ: height, minY: 0, maxY: 1.1 },
          ]) {
            for (const x of [bounds.minX, bounds.maxX]) {
              for (const z of [bounds.minZ, bounds.maxZ]) {
                for (const y of [bounds.minY, bounds.maxY]) {
                  const projected = new Vector3(x, y, z).project(camera)
                  expect(Math.abs(projected.x)).toBeLessThan(1)
                  expect(Math.abs(projected.y)).toBeLessThan(1)
                  expect(Math.abs(projected.z)).toBeLessThan(1)
                }
              }
            }
          }
        }
      }
    },
  )
})
