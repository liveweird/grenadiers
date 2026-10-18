import { Container, Graphics } from 'pixi.js'
import type { Direction, Seat } from './types'
import { PALETTE } from './palette'
import { pixels } from './pixel'

type PoseFrames = readonly [Graphics, Graphics]
export interface ActorDrawing {
  root: Container
  protection: Graphics
  shield: Graphics
  poses: Record<Direction, PoseFrames>
  direction: Direction
  moving: boolean
  protected: boolean
  shielded: boolean
}

function legs(drawing: Graphics, frame: number, side: boolean): void {
  const offset = frame === 0 ? 0 : 3
  const left = side ? 0 : offset
  const right = side ? offset : 0
  pixels(drawing, PALETTE.deepInk, [[-10, 10 + left, 7, 10 - left], [3, 10 + right, 7, 10 - right]])
  pixels(drawing, 0x52606b, [[-9, 10 + left, 5, 4], [4, 10 + right, 5, 4]])
}

function helmetMark(drawing: Graphics, seat: Seat, direction: Direction): void {
  if (seat === 'one') {
    if (direction === 'east' || direction === 'west') pixels(drawing, PALETTE.white, [[-2, -19, 6, 8]])
    else pixels(drawing, PALETTE.white, [[-3, -20, 6, 9], [-6, -17, 12, 3]])
  } else if (direction === 'east' || direction === 'west') {
    pixels(drawing, PALETTE.white, [[-4, -18, 8, 3], [1, -15, 6, 3]])
  } else {
    pixels(drawing, PALETTE.white, [[-7, -18, 14, 3], [-4, -15, 8, 3]])
  }
}

function frontOrBack(seat: Seat, direction: 'north' | 'south', frame: number): Graphics {
  const drawing = new Graphics()
  const uniform = seat === 'one' ? PALETTE.brass : PALETTE.blue
  const light = seat === 'one' ? PALETTE.brassLight : PALETTE.blueLight
  legs(drawing, frame, false)
  pixels(drawing, PALETTE.deepInk, [[-11, -5, 22, 18], [-8, -11, 16, 27]])
  pixels(drawing, uniform, [[-8, -4, 16, 17], [-10, 0, 20, 9]])
  pixels(drawing, light, [[-7, -3, 4, 13], [-9, 1, 3, 6]])
  pixels(drawing, PALETTE.deepInk, [[-13, -14, 26, 12], [-10, -18, 20, 19], [-7, -21, 14, 22]])
  pixels(drawing, uniform, [[-10, -14, 20, 11], [-7, -18, 14, 17]])
  pixels(drawing, light, [[-7, -17, 8, 4], [-9, -13, 4, 7]])
  helmetMark(drawing, seat, direction)
  if (direction === 'south') {
    pixels(drawing, PALETTE.skin, [[-7, -9, 14, 7]])
    pixels(drawing, PALETTE.deepInk, [[-5, -7, 3, 3], [3, -7, 3, 3]])
    pixels(drawing, PALETTE.visor, [[-6, -10, 12, 3]])
  } else {
    pixels(drawing, PALETTE.deepInk, [[-6, -9, 12, 5]])
    pixels(drawing, PALETTE.brickShade, [[-4, 1, 8, 8]])
  }
  return drawing
}

function sidePose(seat: Seat, direction: 'east' | 'west', frame: number): Graphics {
  const drawing = new Graphics()
  const uniform = seat === 'one' ? PALETTE.brass : PALETTE.blue
  const light = seat === 'one' ? PALETTE.brassLight : PALETTE.blueLight
  legs(drawing, frame, true)
  pixels(drawing, PALETTE.deepInk, [[-9, -5, 18, 18], [-7, -11, 16, 27]])
  pixels(drawing, uniform, [[-6, -4, 13, 17], [-8, 0, 18, 8]])
  pixels(drawing, light, [[-5, -3, 4, 13]])
  const front = direction === 'east' ? 1 : -1
  pixels(drawing, PALETTE.deepInk, [[-11, -14, 22, 12], [-8, -19, 16, 19]])
  pixels(drawing, uniform, [[-8, -14, 16, 11], [-5, -18, 11, 16]])
  pixels(drawing, light, [[-5, -17, 5, 4]])
  helmetMark(drawing, seat, direction)
  pixels(drawing, PALETTE.skin, [[front > 0 ? 3 : -8, -10, 7, 7]])
  pixels(drawing, PALETTE.visor, [[front > 0 ? 4 : -10, -12, 8, 3]])
  pixels(drawing, PALETTE.deepInk, [[front > 0 ? 7 : -8, -8, 3, 3]])
  return drawing
}

export function makeActor(seat: Seat): ActorDrawing {
  const root = new Container()
  const shadow = new Graphics()
  pixels(shadow, PALETTE.deepInk, [[-13, 15, 26, 4], [-9, 19, 18, 2]])
  shadow.alpha = 0.3
  const protection = new Graphics()
  pixels(protection, PALETTE.blueLight, [
    [-18, -24, 10, 3], [-21, -21, 3, 10], [8, -24, 10, 3], [18, -21, 3, 10],
    [-21, 11, 3, 10], [-18, 21, 10, 3], [18, 11, 3, 10], [8, 21, 10, 3],
  ])
  pixels(protection, PALETTE.white, [[-18, -21, 3, 5], [15, -21, 3, 5], [-18, 16, 3, 5], [15, 16, 3, 5]])
  protection.visible = false
  const shield = new Graphics()
  pixels(shield, PALETTE.blue, [
    [-15, -27, 30, 3], [-21, -24, 6, 3], [15, -24, 6, 3],
    [-24, -21, 3, 9], [21, -21, 3, 9], [-27, -12, 3, 27], [24, -12, 3, 27],
    [-24, 15, 3, 9], [21, 15, 3, 9], [-21, 24, 6, 3], [15, 24, 6, 3], [-15, 27, 30, 3],
  ])
  pixels(shield, PALETTE.blueLight, [
    [-12, -24, 12, 3], [-24, -9, 3, 12], [21, -9, 3, 12], [0, 24, 12, 3],
  ])
  shield.visible = false
  const poses = {
    north: [frontOrBack(seat, 'north', 0), frontOrBack(seat, 'north', 1)],
    east: [sidePose(seat, 'east', 0), sidePose(seat, 'east', 1)],
    south: [frontOrBack(seat, 'south', 0), frontOrBack(seat, 'south', 1)],
    west: [sidePose(seat, 'west', 0), sidePose(seat, 'west', 1)],
  } satisfies Record<Direction, PoseFrames>
  root.addChild(shadow, protection, shield, ...Object.values(poses).flat())
  const drawing = {
    root, protection, shield, poses, direction: 'south' as Direction, moving: false,
    protected: false, shielded: false,
  }
  setActorPose(drawing, 'south', 0)
  return drawing
}

export function setActorPose(drawing: ActorDrawing, direction: Direction, frame: 0 | 1): void {
  drawing.direction = direction
  for (const poses of Object.values(drawing.poses)) {
    poses[0].visible = false
    poses[1].visible = false
  }
  drawing.poses[direction][frame].visible = true
}
