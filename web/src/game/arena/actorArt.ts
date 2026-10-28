import { Container, Graphics } from 'pixi.js'
import type { Direction, Seat } from './types'
import { PALETTE } from './palette'
import { pixels } from './pixel'

type PoseFrames = readonly [Container, Container]
export interface ActorNodes {
  root: Container
  shadow: Container
  protection: Container
  shield: Container
  stopwatch: Container
  frozen: Container
  goldenFlame: Container
  shortFuse: Container
  shortFlame: Container
  longFuse: Container
  reverseControls: Container
  diarrhea: Container
  slowFoot: Container
  slowBoot: Container
  slowMarkers: readonly Container[]
  piercing: Container
  power: Container
  landmine: Container
  equipment: Container
  kick: Container
  punch: Container
  bracelet: Container
  wallPass: Container
  cloak: Container
  poses: Record<Direction, PoseFrames>
}
export interface ActorDrawing extends ActorNodes {
  direction: Direction
  moving: boolean
  protected: boolean
  shielded: boolean
  stopwatchActive: boolean
  actionsFrozen: boolean
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
  kickEquipped: boolean
  punchEquipped: boolean
  braceletEquipped: boolean
  wallPassEquipped: boolean
  cloaked: boolean
}

function legs(drawing: Graphics, frame: number, side: boolean): void {
  const offset = frame === 0 ? 0 : 3
  const left = side ? 0 : offset
  const right = side ? offset : 0
  pixels(drawing, PALETTE.deepInk, [[-10, 10 + left, 7, 10 - left], [3, 10 + right, 7, 10 - right]])
  pixels(drawing, 0x52606b, [[-9, 10 + left, 5, 4], [4, 10 + right, 5, 4]])
}

function helmetMark(drawing: Graphics, appearance: Seat, direction: Direction): void {
  if (appearance === 'one') {
    if (direction === 'east' || direction === 'west') pixels(drawing, PALETTE.white, [[-2, -19, 6, 8]])
    else pixels(drawing, PALETTE.white, [[-3, -20, 6, 9], [-6, -17, 12, 3]])
  } else if (appearance === 'two' && (direction === 'east' || direction === 'west')) {
    pixels(drawing, PALETTE.white, [[-4, -18, 8, 3], [1, -15, 6, 3]])
  } else if (appearance === 'two') {
    pixels(drawing, PALETTE.white, [[-7, -18, 14, 3], [-4, -15, 8, 3]])
  } else if (appearance === 'three') {
    if (direction === 'east' || direction === 'west') {
      pixels(drawing, PALETTE.silver, [[-5, -19, 3, 8], [1, -16, 6, 3]])
    } else {
      pixels(drawing, PALETTE.silver, [[-7, -19, 14, 3], [-2, -19, 4, 8]])
    }
  } else if (direction === 'east' || direction === 'west') {
    pixels(drawing, PALETTE.gold, [[-5, -18, 10, 3], [-2, -21, 4, 9]])
  } else {
    pixels(drawing, PALETTE.gold, [[-7, -19, 14, 3], [-5, -16, 3, 5], [2, -16, 3, 5]])
  }
}

function uniformColors(appearance: Seat): readonly [number, number] {
  if (appearance === 'one') return [PALETTE.brass, PALETTE.brassLight]
  if (appearance === 'two') return [PALETTE.blue, PALETTE.blueLight]
  if (appearance === 'three') return [PALETTE.forest, PALETTE.forestLight]
  return [PALETTE.plum, PALETTE.plumLight]
}

function frontOrBack(appearance: Seat, direction: 'north' | 'south', frame: number): Graphics {
  const drawing = new Graphics()
  const [uniform, light] = uniformColors(appearance)
  legs(drawing, frame, false)
  pixels(drawing, PALETTE.deepInk, [[-11, -5, 22, 18], [-8, -11, 16, 27]])
  pixels(drawing, uniform, [[-8, -4, 16, 17], [-10, 0, 20, 9]])
  pixels(drawing, light, [[-7, -3, 4, 13], [-9, 1, 3, 6]])
  if (appearance === 'three' || appearance === 'four') {
    pixels(drawing, PALETTE.white, [[2, -3, 3, 13]])
  }
  pixels(drawing, PALETTE.deepInk, [[-13, -14, 26, 12], [-10, -18, 20, 19], [-7, -21, 14, 22]])
  pixels(drawing, uniform, [[-10, -14, 20, 11], [-7, -18, 14, 17]])
  pixels(drawing, light, [[-7, -17, 8, 4], [-9, -13, 4, 7]])
  helmetMark(drawing, appearance, direction)
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

function sidePose(appearance: Seat, direction: 'east' | 'west', frame: number): Graphics {
  const drawing = new Graphics()
  const [uniform, light] = uniformColors(appearance)
  legs(drawing, frame, true)
  pixels(drawing, PALETTE.deepInk, [[-9, -5, 18, 18], [-7, -11, 16, 27]])
  pixels(drawing, uniform, [[-6, -4, 13, 17], [-8, 0, 18, 8]])
  pixels(drawing, light, [[-5, -3, 4, 13]])
  if (appearance === 'three' || appearance === 'four') {
    pixels(drawing, PALETTE.white, [[1, -3, 3, 13]])
  }
  const front = direction === 'east' ? 1 : -1
  pixels(drawing, PALETTE.deepInk, [[-11, -14, 22, 12], [-8, -19, 16, 19]])
  pixels(drawing, uniform, [[-8, -14, 16, 11], [-5, -18, 11, 16]])
  pixels(drawing, light, [[-5, -17, 5, 4]])
  helmetMark(drawing, appearance, direction)
  pixels(drawing, PALETTE.skin, [[front > 0 ? 3 : -8, -10, 7, 7]])
  pixels(drawing, PALETTE.visor, [[front > 0 ? 4 : -10, -12, 8, 3]])
  pixels(drawing, PALETTE.deepInk, [[front > 0 ? 7 : -8, -8, 3, 3]])
  return drawing
}

export function makeActor(appearance: Seat): ActorDrawing {
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
  const shield = new Graphics()
  pixels(shield, PALETTE.blue, [
    [-15, -27, 30, 3], [-21, -24, 6, 3], [15, -24, 6, 3],
    [-24, -21, 3, 9], [21, -21, 3, 9], [-27, -12, 3, 27], [24, -12, 3, 27],
    [-24, 15, 3, 9], [21, 15, 3, 9], [-21, 24, 6, 3], [15, 24, 6, 3], [-15, 27, 30, 3],
  ])
  pixels(shield, PALETTE.blueLight, [
    [-12, -24, 12, 3], [-24, -9, 3, 12], [21, -9, 3, 12], [0, 24, 12, 3],
  ])
  const stopwatch = new Graphics()
  pixels(stopwatch, PALETTE.deepInk, [[-8, -34, 16, 16], [-4, -38, 8, 5]])
  pixels(stopwatch, PALETTE.white, [[-5, -31, 10, 10]])
  pixels(stopwatch, PALETTE.blue, [[-1, -30, 3, 7], [-1, -24, 6, 3]])
  const frozen = new Graphics()
  pixels(frozen, PALETTE.blueLight, [
    [-24, -25, 9, 3], [-24, -25, 3, 9], [15, -25, 9, 3], [21, -25, 3, 9],
    [-24, 20, 9, 3], [-24, 14, 3, 9], [15, 20, 9, 3], [21, 14, 3, 9],
  ])
  const goldenFlame = new Graphics()
  pixels(goldenFlame, PALETTE.deepInk, [
    [-8, -30, 16, 3], [-11, -27, 22, 4], [-8, -23, 16, 3],
  ])
  pixels(goldenFlame, PALETTE.flame, [[-7, -29, 14, 4], [-4, -34, 8, 9], [-1, -38, 4, 9]])
  pixels(goldenFlame, PALETTE.flameLight, [[-3, -31, 6, 7], [0, -35, 2, 6]])
  pixels(goldenFlame, PALETTE.flameCore, [[-1, -29, 3, 5]])
  const shortFuse = new Graphics()
  pixels(shortFuse, PALETTE.deepInk, [[13, -15, 10, 3], [20, -18, 3, 5]])
  pixels(shortFuse, PALETTE.flame, [[22, -23, 4, 5], [26, -20, 4, 4], [18, -23, 3, 3]])
  pixels(shortFuse, PALETTE.flameCore, [[23, -21, 3, 3]])
  const shortFlame = new Graphics()
  pixels(shortFlame, PALETTE.deepInk, [[-25, -22, 12, 3], [-22, -25, 6, 9], [-19, -29, 3, 6]])
  pixels(shortFlame, PALETTE.blue, [[-22, -22, 6, 6], [-19, -26, 3, 7]])
  pixels(shortFlame, PALETTE.blueLight, [[-20, -23, 2, 4]])
  const longFuse = new Graphics()
  pixels(longFuse, PALETTE.deepInk, [[-24, -12, 15, 3], [-12, -15, 3, 5]])
  pixels(longFuse, PALETTE.flame, [[-10, -20, 4, 5], [-6, -17, 4, 4]])
  pixels(longFuse, PALETTE.flameCore, [[-9, -18, 3, 3]])
  const reverseControls = new Graphics()
  pixels(reverseControls, PALETTE.deepInk, [
    [-29, -7, 12, 3], [-29, 5, 12, 3], [17, -7, 12, 3], [17, 5, 12, 3],
    [-29, -10, 3, 9], [26, 2, 3, 9],
  ])
  pixels(reverseControls, PALETTE.white, [
    [-32, -7, 9, 3], [-29, -10, 3, 9], [23, 5, 9, 3], [26, 2, 3, 9],
  ])
  pixels(reverseControls, PALETTE.brick, [[17, -7, 9, 3], [-26, 5, 9, 3]])
  const diarrhea = new Graphics()
  pixels(diarrhea, PALETTE.deepInk, [
    [-14, -29, 28, 3], [-17, -26, 34, 3], [-20, -23, 40, 3],
  ])
  pixels(diarrhea, PALETTE.brick, [[-10, -26, 8, 3], [4, -23, 8, 3]])
  pixels(diarrhea, PALETTE.white, [[-3, -32, 6, 9]])
  const slowFoot = new Container()
  const slowBoot = new Graphics()
  pixels(slowBoot, PALETTE.deepInk, [[-13, 19, 18, 5], [-9, 14, 9, 7], [5, 21, 8, 3]])
  pixels(slowBoot, PALETTE.brick, [[-8, 15, 7, 5], [-11, 20, 14, 3]])
  pixels(slowBoot, PALETTE.white, [[8, 12, 3, 7], [5, 16, 9, 3]])
  const slowMarkers = Array.from({ length: 6 }, (_, index) => {
    const marker = new Graphics()
    pixels(marker, PALETTE.white, [[-14 + index * 5, 27, 3, 3]])
    return marker
  })
  slowFoot.addChild(slowBoot, ...slowMarkers)
  const piercing = new Graphics()
  pixels(piercing, PALETTE.deepInk, [[14, 12, 13, 12], [18, 8, 7, 18]])
  pixels(piercing, PALETTE.white, [[18, 14, 6, 3], [20, 11, 3, 9]])
  pixels(piercing, PALETTE.brick, [[16, 19, 10, 3]])
  const power = new Graphics()
  pixels(power, PALETTE.deepInk, [
    [17, 7, 7, 3], [14, 10, 13, 3], [11, 13, 19, 7], [14, 20, 13, 3], [17, 23, 7, 3],
  ])
  pixels(power, PALETTE.flame, [[17, 11, 7, 3], [14, 14, 13, 5], [17, 19, 7, 3]])
  pixels(power, PALETTE.white, [[19, 13, 3, 7]])
  const landmine = new Graphics()
  pixels(landmine, PALETTE.deepInk, [[-27, 13, 16, 7], [-24, 9, 10, 10]])
  pixels(landmine, PALETTE.brass, [[-24, 12, 10, 5], [-21, 9, 5, 6]])
  pixels(landmine, PALETTE.brick, [[-20, 12, 3, 3]])
  const kick = new Graphics()
  pixels(kick, PALETTE.deepInk, [[-18, 13, 14, 9], [-14, 9, 8, 13]])
  pixels(kick, PALETTE.brassLight, [[-15, 12, 7, 7], [-17, 17, 11, 3]])
  const punch = new Graphics()
  pixels(punch, PALETTE.deepInk, [[11, 10, 15, 12], [14, 6, 4, 8], [19, 5, 4, 9], [24, 8, 4, 9]])
  pixels(punch, PALETTE.brassLight, [[14, 11, 10, 8], [16, 8, 3, 6], [20, 7, 3, 7]])
  const bracelet = new Graphics()
  pixels(bracelet, PALETTE.deepInk, [[-15, -2, 6, 12], [9, -2, 6, 12]])
  pixels(bracelet, PALETTE.blueLight, [[-13, 0, 3, 8], [10, 0, 3, 8]])
  const equipment = new Container()
  equipment.addChild(kick, punch, bracelet)
  const wallPass = new Graphics()
  pixels(wallPass, PALETTE.deepInk, [[-16, -23, 5, 43], [11, -23, 5, 43], [-16, -23, 32, 5]])
  pixels(wallPass, PALETTE.brick, [[-13, -20, 4, 37], [9, -20, 4, 37], [-9, -20, 18, 4]])
  pixels(wallPass, PALETTE.blueLight, [[-9, -16, 3, 30], [6, -16, 3, 30]])
  const cloak = new Graphics()
  pixels(cloak, PALETTE.deepInk, [
    [-15, -26, 30, 3], [-18, -23, 3, 12], [15, -23, 3, 12],
    [-15, -11, 6, 3], [9, -11, 6, 3],
  ])
  pixels(cloak, PALETTE.blue, [[-12, -23, 24, 3], [-15, -20, 3, 9], [12, -20, 3, 9]])
  pixels(cloak, PALETTE.white, [[-9, -20, 6, 3], [6, -20, 3, 3]])
  const poses = {
    north: [frontOrBack(appearance, 'north', 0), frontOrBack(appearance, 'north', 1)],
    east: [sidePose(appearance, 'east', 0), sidePose(appearance, 'east', 1)],
    south: [frontOrBack(appearance, 'south', 0), frontOrBack(appearance, 'south', 1)],
    west: [sidePose(appearance, 'west', 0), sidePose(appearance, 'west', 1)],
  } satisfies Record<Direction, PoseFrames>
  return assembleActorDrawing({
    root, protection, shield, stopwatch, frozen, goldenFlame, shortFuse, shortFlame, longFuse, reverseControls,
    shadow, diarrhea, slowFoot, slowBoot, slowMarkers, piercing, power, landmine, equipment,
    kick, punch, bracelet, wallPass, cloak, poses,
  })
}

export function assembleActorDrawing(nodes: ActorNodes): ActorDrawing {
  for (const cue of [
    nodes.protection, nodes.shield, nodes.stopwatch, nodes.frozen, nodes.goldenFlame,
    nodes.shortFuse, nodes.shortFlame, nodes.longFuse, nodes.reverseControls, nodes.diarrhea,
    nodes.slowFoot, nodes.piercing, nodes.power, nodes.landmine, nodes.equipment,
    nodes.kick, nodes.punch, nodes.bracelet, nodes.wallPass, nodes.cloak,
  ]) cue.visible = false
  for (const marker of nodes.slowMarkers) marker.visible = false
  const drawing: ActorDrawing = {
    ...nodes,
    direction: 'south' as Direction, moving: false,
    protected: false, shielded: false, stopwatchActive: false, actionsFrozen: false,
    golden: false, shortFused: false, shortFlamed: false,
    longFused: false, reversed: false, diarrheaActive: false, slowFootStacks: 0,
    piercingArmed: false, powerArmed: false, landmineStored: false, kickEquipped: false,
    punchEquipped: false, braceletEquipped: false, wallPassEquipped: false, cloaked: false,
  }
  const { root } = nodes
  root.addChild(
    nodes.shadow, nodes.protection, nodes.shield, nodes.goldenFlame, nodes.shortFuse,
    nodes.shortFlame, nodes.longFuse, nodes.reverseControls, nodes.diarrhea, nodes.slowFoot,
    nodes.piercing, nodes.power, nodes.landmine, nodes.equipment,
    ...Object.values(nodes.poses).flat(), nodes.cloak, nodes.stopwatch, nodes.frozen, nodes.wallPass,
  )
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
