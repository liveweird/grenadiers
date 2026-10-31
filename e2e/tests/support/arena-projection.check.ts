import assert from 'node:assert/strict'
import { projectedCellFootprint } from './arena-projection.ts'

const wideCenter = projectedCellFootprint({ width: 1200, height: 500 }, 7, 6)
assert.deepEqual(wideCenter, { x: 585, y: 215, width: 30, height: 44 })
assert(wideCenter.x < 600 && wideCenter.x + wideCenter.width > 600)
assert(wideCenter.y < 250 && wideCenter.y + wideCenter.height > 250)

const near = projectedCellFootprint({ width: 1200, height: 500 }, 7, 7)
assert.deepEqual(near, { x: 585, y: 243, width: 30, height: 44 })
assert(near.y > wideCenter.y, 'rows nearer the no-yaw camera must project lower')
assert.equal(near.x, wideCenter.x, 'rows must not introduce yaw')

const right = projectedCellFootprint({ width: 1200, height: 500 }, 8, 6)
assert(right.x > wideCenter.x)
assert.equal(right.y, wideCenter.y)

const narrow = projectedCellFootprint({ width: 320, height: 640 }, 7, 6)
assert.deepEqual(narrow, { x: 153, y: 303, width: 14, height: 22 })
assert(narrow.x < 160 && narrow.x + narrow.width > 160)
assert(narrow.width < wideCenter.width)
assert(narrow.height < wideCenter.height)

const corner = projectedCellFootprint({ width: 320, height: 640 }, 0, 0)
assert(corner.x >= 0 && corner.y >= 0)
assert(corner.x + corner.width <= 320 && corner.y + corner.height <= 640)

console.log('THREE_ARENA_PROJECTION_PASS')
