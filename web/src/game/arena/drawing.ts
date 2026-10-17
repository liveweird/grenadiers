import { Container, Graphics } from 'pixi.js'
import type { PickupKind } from '../schema'
import type { Direction, Seat, SnapshotMessage } from './types'

export const TILE = 48
const INK = 0x20354a
export const GROUND = 0xb7d0c3

export function paintTerrain(graphics: Graphics, arena: SnapshotMessage['state']['arena']): void {
  graphics.clear()
  for (const [index, tile] of arena.terrain.entries()) {
    const column = index % arena.width
    const row = Math.floor(index / arena.width)
    const x = column * TILE
    const y = row * TILE
    if (tile === 'wall') {
      graphics.rect(x, y, TILE, TILE).fill(INK)
        .roundRect(x + 2, y + 2, TILE - 4, TILE - 7, 4).fill(0x778b9a)
        .rect(x + 6, y + 5, TILE - 12, 3).fill(0xaabcc6)
    } else {
      graphics.rect(x, y, TILE, TILE).fill(GROUND)
        .rect(x + 1, y + 1, TILE - 2, TILE - 2)
        .fill((column + row) % 2 === 0 ? 0xc4d9ce : 0xb8d0c2)
    }
  }
}

export function makeBlock(): Graphics {
  return new Graphics().roundRect(2, 2, 44, 44, 3).fill(INK)
    .roundRect(4, 3, 40, 38, 2).fill(0xa9694f)
    .rect(6, 5, 36, 3).fill(0xd29672)
    .moveTo(4, 16).lineTo(44, 16).moveTo(4, 29).lineTo(44, 29)
    .moveTo(25, 4).lineTo(25, 16).moveTo(17, 16).lineTo(17, 29)
    .moveTo(30, 29).lineTo(30, 41).stroke({ color: 0x704a40, width: 3 })
}

export function makeBomb(): Container {
  const root = new Container()
  root.addChild(new Graphics().ellipse(0, 11, 17, 7).fill({ color: INK, alpha: 0.25 }))
  root.addChild(new Graphics().circle(0, 0, 14).fill(INK)
    .circle(-4, -5, 4).fill(0x748b9c)
    .moveTo(7, -11).lineTo(10, -20).lineTo(16, -18)
    .stroke({ color: 0xf3db9d, width: 3 })
    .poly([16, -24, 18, -19, 23, -18, 18, -16, 16, -11, 14, -17, 10, -19, 14, -20])
    .fill(0xf7b334))
  return root
}

export function makeFlame(): Graphics {
  return new Graphics().roundRect(3, 14, 42, 20, 8).fill(0xf27831)
    .roundRect(14, 3, 20, 42, 8).fill(0xf27831)
    .roundRect(7, 19, 34, 10, 5).fill(0xffe29a)
    .roundRect(19, 7, 10, 34, 5).fill(0xffe29a)
}

export function makePickup(kind: PickupKind): Graphics {
  const drawing = new Graphics()
    .ellipse(24, 39, 15, 4).fill({ color: INK, alpha: 0.2 })
    .roundRect(8, 7, 32, 32, 8).fill(0xf8fbfa)
    .stroke({ color: INK, width: 2 })
  if (kind === 'bombCapacity') {
    return drawing.circle(23, 24, 10).fill(INK)
      .circle(20, 21, 3).fill(0x778b9a)
      .moveTo(29, 16).lineTo(33, 11).lineTo(37, 13)
      .stroke({ color: 0xedb94b, width: 3 })
  }
  if (kind === 'blastRange') {
    return drawing.roundRect(20, 12, 8, 24, 3).fill(0xf27831)
      .roundRect(12, 20, 24, 8, 3).fill(0xf27831)
      .circle(24, 24, 4).fill(0xffe29a)
  }
  return drawing
    .poly([12, 18, 24, 18, 19, 12, 23, 12, 31, 22, 23, 32, 19, 32, 24, 26, 12, 26])
    .fill(0x385fa8)
    .poly([27, 17, 36, 22, 27, 27]).fill(0xedb94b)
}

export interface ActorDrawing {
  root: Container
  body: Graphics
}

export function makeActor(seat: Seat): ActorDrawing {
  const root = new Container()
  root.addChild(new Graphics().ellipse(0, 13, 15, 6).fill({ color: INK, alpha: 0.25 }))
  const body = new Graphics().roundRect(-10, -2, 20, 22, 6)
    .fill(seat === 'one' ? 0xedb94b : 0x385fa8).stroke({ color: INK, width: 2 })
    .roundRect(-12, 12, 9, 8, 3).fill(INK)
    .roundRect(3, 12, 9, 8, 3).fill(INK)
    .circle(0, -8, 13).fill(seat === 'one' ? 0xedb94b : 0x385fa8)
    .stroke({ color: INK, width: 2 })
    .roundRect(-9, -5, 18, 8, 3).fill(0xf6e9c7)
    .circle(-4, -1, 2).fill(INK).circle(4, -1, 2).fill(INK)
  if (seat === 'one') body.circle(0, -13, 4).fill(0xffffff)
  else body.poly([-5, -16, 5, -16, 0, -9]).fill(0xffffff)
  root.addChild(body)
  return { root, body }
}

export function headingRotation(direction: Direction): number {
  return { north: Math.PI, east: -Math.PI / 2, south: 0, west: Math.PI / 2 }[direction]
}
