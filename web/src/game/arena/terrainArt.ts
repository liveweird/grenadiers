import { Graphics } from 'pixi.js'
import type { SnapshotMessage } from './types'
import { PALETTE } from './palette'
import { deterministicVariant, pixels, type PixelRect } from './pixel'

export const TILE = 48
export const GROUND = PALETTE.floor

const FLOOR_CHIPS: readonly (readonly PixelRect[])[] = [
  [[9, 12, 6, 3], [12, 15, 3, 3], [34, 35, 5, 3]],
  [[30, 9, 6, 3], [27, 12, 3, 6], [10, 34, 3, 3]],
  [[8, 28, 8, 3], [15, 25, 3, 6], [35, 16, 3, 6]],
  [[24, 35, 9, 3], [30, 32, 3, 3], [10, 10, 4, 3]],
]

function paintFloorPattern(
  graphics: Graphics, x: number, y: number, chipVariant: number, moss: boolean,
): void {
  graphics.rect(x, y, TILE, TILE).fill(PALETTE.floorShade)
  graphics.rect(x + 2, y + 2, 44, 44).fill(PALETTE.floor)
  graphics.rect(x + 3, y + 3, 42, 3).fill(PALETTE.floorLight)
  graphics.rect(x + 3, y + 42, 42, 3).fill(0xaeb69a)
  pixels(graphics, chipVariant % 2 === 0 ? PALETTE.floorLight : PALETTE.floorShade,
    FLOOR_CHIPS[chipVariant].map(([chipX, chipY, width, height]) => [x + chipX, y + chipY, width, height]))
  if (moss) {
    pixels(graphics, PALETTE.moss, [[x + 3, y + 39, 9, 3], [x + 6, y + 36, 6, 3]])
  }
}

function paintFloor(graphics: Graphics, x: number, y: number, column: number, row: number): void {
  paintFloorPattern(
    graphics, x, y, deterministicVariant(column, row, FLOOR_CHIPS.length),
    (column * 5 + row * 3) % 7 === 0,
  )
}

export function makeFloorVariant(variant: number): Graphics {
  if (!Number.isInteger(variant) || variant < 0 || variant >= 8) {
    throw new RangeError('floor variant must be an integer from 0 through 7')
  }
  const drawing = new Graphics()
  paintFloorPattern(drawing, 0, 0, variant % FLOOR_CHIPS.length, variant >= FLOOR_CHIPS.length)
  return drawing
}

function paintWall(graphics: Graphics, x: number, y: number, column: number, row: number): void {
  graphics.rect(x, y, TILE, TILE).fill(PALETTE.deepInk)
  graphics.rect(x + 2, y + 2, 44, 43).fill(PALETTE.ink)
  pixels(graphics, 0x667366, [
    [x + 5, y + 7, 17, 12], [x + 25, y + 7, 18, 12],
    [x + 5, y + 22, 10, 13], [x + 18, y + 22, 18, 13], [x + 39, y + 22, 4, 13],
  ])
  pixels(graphics, 0x879181, [
    [x + 5, y + 7, 17, 3], [x + 25, y + 7, 18, 3],
    [x + 5, y + 22, 10, 3], [x + 18, y + 22, 18, 3], [x + 39, y + 22, 4, 3],
  ])
  pixels(graphics, PALETTE.floorLight, [[x + 4, y + 3, 40, 3], [x + 7, y + 39, 34, 3]])
  pixels(graphics, PALETTE.brass, [[x + 7, y + 4, 3, 3], [x + 38, y + 4, 3, 3]])
  if (deterministicVariant(column, row, 3) === 0) {
    pixels(graphics, PALETTE.moss, [[x + 5, y + 34, 10, 3], [x + 8, y + 31, 4, 3]])
  }
}

export function makeWallVariant(variant: number): Graphics {
  if (!Number.isInteger(variant) || variant < 0 || variant >= 2) {
    throw new RangeError('wall variant must be 0 or 1')
  }
  const drawing = new Graphics()
  paintWall(drawing, 0, 0, variant === 0 ? 1 : 0, 0)
  return drawing
}

export function paintTerrain(graphics: Graphics, arena: SnapshotMessage['state']['arena']): void {
  graphics.clear()
  for (const [index, tile] of arena.terrain.entries()) {
    const column = index % arena.width
    const row = Math.floor(index / arena.width)
    const x = column * TILE
    const y = row * TILE
    if (tile === 'wall') paintWall(graphics, x, y, column, row)
    else paintFloor(graphics, x, y, column, row)
  }
}

export function makeBlock(): Graphics {
  const drawing = new Graphics()
  pixels(drawing, PALETTE.deepInk, [[2, 3, 44, 42]])
  pixels(drawing, PALETTE.brickShade, [[4, 5, 40, 37]])
  pixels(drawing, PALETTE.brick, [
    [5, 6, 17, 9], [25, 6, 18, 9], [5, 18, 10, 9], [18, 18, 18, 9], [39, 18, 4, 9],
    [5, 30, 17, 9], [25, 30, 18, 9],
  ])
  pixels(drawing, PALETTE.brickLight, [
    [6, 7, 15, 3], [26, 7, 16, 3], [6, 19, 8, 3], [19, 19, 16, 3],
    [6, 31, 15, 3], [26, 31, 16, 3],
  ])
  pixels(drawing, PALETTE.floorLight, [[5, 4, 38, 2]])
  pixels(drawing, PALETTE.moss, [[7, 39, 13, 3], [10, 36, 4, 3]])
  return drawing
}
