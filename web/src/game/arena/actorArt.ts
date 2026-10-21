import { Container, Graphics } from 'pixi.js'
import type { Direction, Seat } from './types'
import { PALETTE } from './palette'
import { pixels } from './pixel'

type PoseFrames = readonly [Graphics, Graphics]
export interface ActorDrawing {
  root: Container
  protection: Graphics
  shield: Graphics
  goldenFlame: Graphics
  shortFuse: Graphics
  shortFlame: Graphics
  longFuse: Graphics
  reverseControls: Graphics
  diarrhea: Graphics
  slowFoot: Container
  piercing: Graphics
  power: Graphics
  landmine: Graphics
  cloak: Graphics
  poses: Record<Direction, PoseFrames>
  direction: Direction
  moving: boolean
  protected: boolean
  shielded: boolean
  golden: boolean
  shortFused: boolean
  shortFlamed: boolean
  longFused: boolean
  reversed: boolean
  diarrheaActive: boolean
  slowFootStacks: number
  piercingArmed: boolean
  powerArmed: boolean
  landmineStored: boolean
  cloaked: boolean
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
  const goldenFlame = new Graphics()
  pixels(goldenFlame, PALETTE.deepInk, [
    [-8, -30, 16, 3], [-11, -27, 22, 4], [-8, -23, 16, 3],
  ])
  pixels(goldenFlame, PALETTE.flame, [[-7, -29, 14, 4], [-4, -34, 8, 9], [-1, -38, 4, 9]])
  pixels(goldenFlame, PALETTE.flameLight, [[-3, -31, 6, 7], [0, -35, 2, 6]])
  pixels(goldenFlame, PALETTE.flameCore, [[-1, -29, 3, 5]])
  goldenFlame.visible = false
  const shortFuse = new Graphics()
  pixels(shortFuse, PALETTE.deepInk, [[13, -15, 10, 3], [20, -18, 3, 5]])
  pixels(shortFuse, PALETTE.flame, [[22, -23, 4, 5], [26, -20, 4, 4], [18, -23, 3, 3]])
  pixels(shortFuse, PALETTE.flameCore, [[23, -21, 3, 3]])
  shortFuse.visible = false
  const shortFlame = new Graphics()
  pixels(shortFlame, PALETTE.deepInk, [[-25, -22, 12, 3], [-22, -25, 6, 9], [-19, -29, 3, 6]])
  pixels(shortFlame, PALETTE.blue, [[-22, -22, 6, 6], [-19, -26, 3, 7]])
  pixels(shortFlame, PALETTE.blueLight, [[-20, -23, 2, 4]])
  shortFlame.visible = false
  const longFuse = new Graphics()
  pixels(longFuse, PALETTE.deepInk, [[-24, -12, 15, 3], [-12, -15, 3, 5]])
  pixels(longFuse, PALETTE.flame, [[-10, -20, 4, 5], [-6, -17, 4, 4]])
  pixels(longFuse, PALETTE.flameCore, [[-9, -18, 3, 3]])
  longFuse.visible = false
  const reverseControls = new Graphics()
  pixels(reverseControls, PALETTE.deepInk, [
    [-29, -7, 12, 3], [-29, 5, 12, 3], [17, -7, 12, 3], [17, 5, 12, 3],
    [-29, -10, 3, 9], [26, 2, 3, 9],
  ])
  pixels(reverseControls, PALETTE.white, [
    [-32, -7, 9, 3], [-29, -10, 3, 9], [23, 5, 9, 3], [26, 2, 3, 9],
  ])
  pixels(reverseControls, PALETTE.brick, [[17, -7, 9, 3], [-26, 5, 9, 3]])
  reverseControls.visible = false
  const diarrhea = new Graphics()
  pixels(diarrhea, PALETTE.deepInk, [
    [-14, -29, 28, 3], [-17, -26, 34, 3], [-20, -23, 40, 3],
  ])
  pixels(diarrhea, PALETTE.brick, [[-10, -26, 8, 3], [4, -23, 8, 3]])
  pixels(diarrhea, PALETTE.white, [[-3, -32, 6, 9]])
  diarrhea.visible = false
  const slowFoot = new Container()
  const slowBoot = new Graphics()
  pixels(slowBoot, PALETTE.deepInk, [[-13, 19, 18, 5], [-9, 14, 9, 7], [5, 21, 8, 3]])
  pixels(slowBoot, PALETTE.brick, [[-8, 15, 7, 5], [-11, 20, 14, 3]])
  pixels(slowBoot, PALETTE.white, [[8, 12, 3, 7], [5, 16, 9, 3]])
  const slowMarkers = Array.from({ length: 6 }, (_, index) => {
    const marker = new Graphics()
    pixels(marker, PALETTE.white, [[-14 + index * 5, 27, 3, 3]])
    marker.visible = false
    return marker
  })
  slowFoot.addChild(slowBoot, ...slowMarkers)
  slowFoot.visible = false
  const piercing = new Graphics()
  pixels(piercing, PALETTE.deepInk, [[14, 12, 13, 12], [18, 8, 7, 18]])
  pixels(piercing, PALETTE.white, [[18, 14, 6, 3], [20, 11, 3, 9]])
  pixels(piercing, PALETTE.brick, [[16, 19, 10, 3]])
  piercing.visible = false
  const power = new Graphics()
  pixels(power, PALETTE.deepInk, [
    [17, 7, 7, 3], [14, 10, 13, 3], [11, 13, 19, 7], [14, 20, 13, 3], [17, 23, 7, 3],
  ])
  pixels(power, PALETTE.flame, [[17, 11, 7, 3], [14, 14, 13, 5], [17, 19, 7, 3]])
  pixels(power, PALETTE.white, [[19, 13, 3, 7]])
  power.visible = false
  const landmine = new Graphics()
  pixels(landmine, PALETTE.deepInk, [[-27, 13, 16, 7], [-24, 9, 10, 10]])
  pixels(landmine, PALETTE.brass, [[-24, 12, 10, 5], [-21, 9, 5, 6]])
  pixels(landmine, PALETTE.brick, [[-20, 12, 3, 3]])
  landmine.visible = false
  const cloak = new Graphics()
  pixels(cloak, PALETTE.deepInk, [
    [-15, -26, 30, 3], [-18, -23, 3, 12], [15, -23, 3, 12],
    [-15, -11, 6, 3], [9, -11, 6, 3],
  ])
  pixels(cloak, PALETTE.blue, [[-12, -23, 24, 3], [-15, -20, 3, 9], [12, -20, 3, 9]])
  pixels(cloak, PALETTE.white, [[-9, -20, 6, 3], [6, -20, 3, 3]])
  cloak.visible = false
  const poses = {
    north: [frontOrBack(seat, 'north', 0), frontOrBack(seat, 'north', 1)],
    east: [sidePose(seat, 'east', 0), sidePose(seat, 'east', 1)],
    south: [frontOrBack(seat, 'south', 0), frontOrBack(seat, 'south', 1)],
    west: [sidePose(seat, 'west', 0), sidePose(seat, 'west', 1)],
  } satisfies Record<Direction, PoseFrames>
  root.addChild(
    shadow, protection, shield, goldenFlame, shortFuse, shortFlame, longFuse, reverseControls,
    diarrhea, slowFoot, piercing, power, landmine,
    ...Object.values(poses).flat(), cloak,
  )
  const drawing = {
    root, protection, shield, goldenFlame, shortFuse, shortFlame, longFuse, reverseControls,
    diarrhea, slowFoot, piercing, power, landmine, cloak, poses,
    direction: 'south' as Direction, moving: false,
    protected: false, shielded: false, golden: false, shortFused: false, shortFlamed: false,
    longFused: false, reversed: false, diarrheaActive: false, slowFootStacks: 0,
    piercingArmed: false, powerArmed: false, landmineStored: false, cloaked: false,
  }
  setActorPose(drawing, 'south', 0)
  return drawing
}

export function setActorPose(drawing: ActorDrawing, direction: Direction, frame: 0 | 1): void {
  drawing.direction = direction
  for (const [poseDirection, poses] of Object.entries(drawing.poses)) {
    poses[0].visible = poseDirection === direction && frame === 0
    poses[1].visible = poseDirection === direction && frame === 1
  }
}
