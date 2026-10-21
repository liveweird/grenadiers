import { Container, Graphics } from 'pixi.js'
import type { RenderablePickupKind } from './types'
import { PALETTE } from './palette'
import { pixels } from './pixel'

export interface BombDrawing {
  root: Container; spark: Graphics; pause: Graphics; remote: boolean; contact: boolean; paused: boolean
}
export interface FlameDrawing { root: Container; frames: readonly [Graphics, Graphics] }

export function makeBomb(
  blast: 'standard' | 'piercing' | 'power' = 'standard',
  trigger: 'timed' | 'remote' | 'contact' = 'timed',
): BombDrawing {
  const root = new Container()
  const shadow = new Graphics()
  pixels(shadow, PALETTE.deepInk, [[-15, 12, 30, 5], [-10, 17, 20, 2]])
  shadow.alpha = 0.28
  const shell = new Graphics()
  pixels(shell, PALETTE.deepInk, [[-12, -9, 24, 21], [-9, -13, 18, 29], [-15, -3, 30, 12]])
  pixels(shell, PALETTE.ink, [[-9, -9, 18, 21], [-12, -3, 24, 12]])
  pixels(shell, 0x536272, [[-7, -9, 6, 9], [-10, -3, 6, 5]])
  pixels(shell, 0x8796a0, [[-6, -8, 3, 4]])
  if (blast === 'piercing') {
    pixels(shell, PALETTE.white, [[-4, -5, 8, 3], [-1, -8, 3, 9]])
    pixels(shell, PALETTE.brick, [[-7, -2, 14, 3]])
  } else if (blast === 'power') {
    pixels(shell, PALETTE.flame, [[-7, -3, 14, 6], [-4, -7, 8, 14]])
    pixels(shell, PALETTE.white, [[-2, -5, 4, 10]])
  }
  if (trigger === 'contact') {
    shell.clear()
    pixels(shell, PALETTE.deepInk, [[-18, 2, 36, 10], [-14, -3, 28, 16]])
    pixels(shell, PALETTE.brass, [[-12, 0, 24, 8], [-8, -5, 16, 7]])
    pixels(shell, PALETTE.brassLight, [[-7, -3, 14, 3]])
    pixels(shell, PALETTE.brick, [[-3, 1, 6, 5]])
  } else if (trigger === 'remote') {
    pixels(shell, PALETTE.brass, [[4, -15, 7, 5], [8, -21, 4, 9], [6, -24, 3, 5], [11, -27, 3, 8]])
    pixels(shell, PALETTE.brick, [[9, -29, 7, 4]])
  } else {
    pixels(shell, PALETTE.brass, [[4, -15, 7, 5], [8, -19, 4, 7], [11, -21, 7, 3]])
  }
  const spark = new Graphics()
  pixels(spark, PALETTE.flameCore, [[16, -25, 4, 4], [21, -21, 4, 4], [12, -21, 4, 4]])
  pixels(spark, PALETTE.flame, [[18, -29, 3, 4], [24, -24, 3, 3]])
  spark.visible = trigger === 'timed'
  const pause = new Graphics()
  pixels(pause, PALETTE.blueLight, [[-8, -7, 5, 15], [3, -7, 5, 15]])
  pause.visible = false
  root.addChild(shadow, shell, spark, pause)
  return { root, spark, pause, remote: trigger === 'remote', contact: trigger === 'contact', paused: false }
}

function flameFrame(alternate: boolean): Graphics {
  const drawing = new Graphics()
  const shift = alternate ? 2 : 0
  pixels(drawing, PALETTE.flame, [
    [0, 18, 48, 15], [12, 9 + shift, 24, 31 - shift], [18, shift, 12, 48 - shift],
    [6, 15, 9, 21], [33, 15, 9, 21],
  ])
  pixels(drawing, PALETTE.flameLight, [
    [7, 21, 34, 9], [18, 9 + shift, 12, 30 - shift], [12, 18, 24, 15],
  ])
  pixels(drawing, PALETTE.flameCore, [[14, 23, 20, 5], [21, 13 + shift, 6, 22 - shift]])
  pixels(drawing, PALETTE.white, [[22, 21, 4, 5]])
  return drawing
}

export function makeFlame(): FlameDrawing {
  const root = new Container()
  const frames = [flameFrame(false), flameFrame(true)] as const
  frames[1].visible = false
  root.addChild(...frames)
  return { root, frames }
}

function pickupFrame(): Graphics {
  const drawing = new Graphics()
  pixels(drawing, PALETTE.deepInk, [[7, 8, 34, 32], [10, 5, 28, 38], [4, 12, 40, 24]])
  pixels(drawing, PALETTE.brass, [[8, 11, 32, 26], [11, 8, 26, 32]])
  pixels(drawing, PALETTE.brassLight, [[11, 10, 24, 4], [8, 14, 4, 18]])
  pixels(drawing, PALETTE.brickShade, [[12, 34, 24, 4], [36, 14, 4, 20]])
  pixels(drawing, PALETTE.white, [[13, 13, 22, 20]])
  return drawing
}

function makeKickPickup(): Graphics {
  const drawing = pickupFrame()
  pixels(drawing, PALETTE.deepInk, [[13, 15, 14, 20], [24, 25, 12, 10]])
  pixels(drawing, PALETTE.brassLight, [[16, 17, 8, 15], [23, 28, 10, 4]])
  pixels(drawing, PALETTE.white, [[17, 18, 4, 8]])
  return drawing
}

function makePunchPickup(): Graphics {
  const drawing = pickupFrame()
  pixels(drawing, PALETTE.deepInk, [[14, 17, 21, 15], [18, 13, 5, 9], [24, 12, 5, 10], [30, 15, 5, 10]])
  pixels(drawing, PALETTE.brassLight, [[17, 19, 15, 10], [20, 15, 3, 7], [25, 14, 3, 8]])
  pixels(drawing, PALETTE.white, [[18, 20, 5, 4]])
  return drawing
}

function makeBraceletPickup(): Graphics {
  const drawing = pickupFrame()
  pixels(drawing, PALETTE.deepInk, [[13, 15, 23, 5], [10, 19, 6, 15], [33, 19, 6, 15], [14, 32, 21, 5]])
  pixels(drawing, PALETTE.blueLight, [[16, 17, 17, 3], [13, 21, 3, 10], [35, 21, 3, 10], [17, 33, 15, 3]])
  pixels(drawing, PALETTE.white, [[17, 18, 7, 2], [14, 22, 2, 6]])
  return drawing
}

function makeWallPassPickup(): Graphics {
  const drawing = pickupFrame()
  pixels(drawing, PALETTE.deepInk, [[13, 14, 23, 23], [17, 18, 15, 19]])
  pixels(drawing, PALETTE.brick, [[15, 16, 19, 5], [15, 23, 6, 12], [28, 23, 6, 12]])
  pixels(drawing, PALETTE.brickLight, [[17, 17, 7, 3], [17, 24, 3, 8]])
  pixels(drawing, PALETTE.blueLight, [[22, 22, 5, 13], [25, 25, 5, 7]])
  return drawing
}

function makeLegacyPickup(kind: Exclude<RenderablePickupKind, 'kick' | 'wallPass'>): Graphics {
  const drawing = pickupFrame()
  if (kind === 'bombCapacity') {
    pixels(drawing, PALETTE.deepInk, [[17, 18, 15, 13], [20, 15, 9, 19], [14, 21, 21, 7]])
    pixels(drawing, 0x647384, [[18, 18, 6, 6]])
    pixels(drawing, PALETTE.flame, [[30, 13, 4, 4], [34, 10, 3, 3]])
  } else if (kind === 'blastRange') {
    pixels(drawing, PALETTE.flame, [[14, 21, 20, 7], [21, 14, 7, 20]])
    pixels(drawing, PALETTE.flameLight, [[18, 23, 12, 3], [23, 18, 3, 12]])
  } else if (kind === 'movementSpeed') {
    pixels(drawing, PALETTE.blue, [[13, 18, 14, 7], [18, 14, 10, 15], [24, 21, 11, 6], [29, 18, 7, 5]])
    pixels(drawing, PALETTE.blueLight, [[16, 17, 9, 3], [29, 20, 4, 3]])
    pixels(drawing, PALETTE.deepInk, [[12, 28, 14, 4], [28, 27, 9, 4]])
  } else if (kind === 'heart') {
    pixels(drawing, PALETTE.deepInk, [
      [15, 16, 8, 4], [26, 16, 8, 4], [12, 20, 25, 8],
      [15, 28, 19, 4], [19, 32, 11, 4], [22, 36, 5, 4],
    ])
    pixels(drawing, PALETTE.brick, [
      [17, 19, 5, 5], [27, 19, 5, 5], [15, 23, 19, 5], [18, 28, 13, 4], [21, 32, 7, 4],
    ])
    pixels(drawing, PALETTE.brickLight, [[18, 20, 4, 3], [16, 23, 4, 3]])
  } else if (kind === 'remoteBomb') {
    pixels(drawing, PALETTE.deepInk, [[16, 22, 17, 13], [20, 18, 9, 20], [28, 14, 4, 8]])
    pixels(drawing, PALETTE.ink, [[19, 23, 11, 10], [22, 20, 6, 5]])
    pixels(drawing, PALETTE.brass, [[29, 12, 3, 7], [26, 9, 3, 6], [31, 7, 3, 7]])
    pixels(drawing, PALETTE.brick, [[29, 5, 7, 4]])
  } else if (kind === 'piercingBomb') {
    pixels(drawing, PALETTE.deepInk, [[15, 21, 19, 14], [19, 17, 11, 21]])
    pixels(drawing, PALETTE.ink, [[18, 22, 13, 11], [21, 19, 7, 5]])
    pixels(drawing, PALETTE.white, [[21, 24, 7, 3], [23, 21, 3, 9]])
    pixels(drawing, PALETTE.brick, [[17, 28, 15, 3]])
  } else if (kind === 'powerBomb') {
    pixels(drawing, PALETTE.deepInk, [
      [21, 13, 7, 3], [17, 16, 15, 3], [14, 19, 21, 11], [17, 30, 15, 3], [21, 33, 7, 3],
    ])
    pixels(drawing, PALETTE.flame, [[21, 17, 7, 3], [18, 20, 13, 9], [21, 29, 7, 3]])
    pixels(drawing, PALETTE.white, [[23, 21, 3, 7]])
  } else if (kind === 'landmine') {
    pixels(drawing, PALETTE.deepInk, [[12, 25, 25, 9], [16, 20, 17, 15]])
    pixels(drawing, PALETTE.brass, [[15, 24, 19, 7], [19, 20, 11, 8]])
    pixels(drawing, PALETTE.brassLight, [[20, 21, 9, 3]])
    pixels(drawing, PALETTE.brick, [[22, 26, 5, 4]])
  } else if (kind === 'shield') {
    pixels(drawing, PALETTE.deepInk, [
      [16, 15, 17, 3], [13, 18, 23, 7], [15, 25, 19, 7], [18, 32, 13, 4], [22, 36, 5, 3],
    ])
    pixels(drawing, PALETTE.blue, [
      [17, 18, 15, 4], [16, 22, 17, 6], [18, 28, 13, 4], [21, 32, 7, 3],
    ])
    pixels(drawing, PALETTE.blueLight, [[18, 19, 9, 3], [18, 22, 4, 5]])
  } else if (kind === 'stopwatch') {
    pixels(drawing, PALETTE.deepInk, [[16, 14, 17, 21], [20, 10, 9, 5]])
    pixels(drawing, PALETTE.white, [[19, 17, 11, 15]])
    pixels(drawing, PALETTE.blue, [[23, 19, 3, 9], [23, 26, 6, 3]])
  } else if (kind === 'invisibilityCloak') {
    pixels(drawing, PALETTE.deepInk, [
      [16, 14, 17, 4], [13, 18, 23, 5], [16, 23, 17, 13], [12, 34, 9, 4], [28, 34, 9, 4],
    ])
    pixels(drawing, PALETTE.blue, [
      [17, 18, 15, 5], [19, 23, 11, 11], [15, 31, 7, 4], [27, 31, 7, 4],
    ])
    pixels(drawing, PALETTE.white, [[20, 19, 9, 3], [22, 24, 5, 7]])
  } else if (kind === 'goldenFlame') {
    pixels(drawing, PALETTE.deepInk, [
      [16, 32, 17, 4], [13, 25, 23, 7], [17, 17, 15, 10], [22, 12, 7, 8],
    ])
    pixels(drawing, PALETTE.flame, [[16, 25, 17, 6], [19, 18, 11, 10], [23, 13, 5, 9]])
    pixels(drawing, PALETTE.flameLight, [[20, 24, 9, 5], [22, 18, 6, 9], [24, 15, 3, 7]])
    pixels(drawing, PALETTE.flameCore, [[23, 22, 4, 7], [24, 18, 2, 5]])
  } else if (kind === 'shortFuse') {
    pixels(drawing, PALETTE.deepInk, [
      [15, 22, 18, 13], [18, 18, 12, 20], [29, 16, 4, 8], [32, 13, 5, 5],
    ])
    pixels(drawing, PALETTE.ink, [[18, 22, 12, 12], [21, 19, 7, 4]])
    pixels(drawing, PALETTE.blueLight, [[19, 23, 4, 5]])
    pixels(drawing, PALETTE.flame, [[35, 10, 4, 5], [38, 14, 4, 4], [31, 10, 3, 3]])
    pixels(drawing, PALETTE.flameCore, [[36, 12, 3, 3]])
  } else if (kind === 'diarrhea') {
    pixels(drawing, PALETTE.deepInk, [[16, 24, 17, 12], [19, 20, 11, 18]])
    pixels(drawing, PALETTE.ink, [[19, 24, 11, 10], [22, 21, 6, 5]])
    pixels(drawing, PALETTE.blueLight, [[20, 25, 4, 5]])
    pixels(drawing, PALETTE.brick, [[12, 17, 8, 3], [20, 14, 8, 3], [28, 11, 8, 3]])
    pixels(drawing, PALETTE.white, [[14, 14, 3, 6], [30, 8, 3, 6]])
  } else if (kind === 'slowFoot') {
    pixels(drawing, PALETTE.deepInk, [[13, 24, 22, 9], [17, 17, 12, 10], [29, 29, 8, 4]])
    pixels(drawing, PALETTE.brick, [[18, 19, 9, 7], [15, 26, 18, 5]])
    pixels(drawing, PALETTE.white, [[22, 11, 4, 7], [18, 14, 12, 4], [23, 32, 3, 5]])
  } else if (kind === 'shortFlame') {
    pixels(drawing, PALETTE.deepInk, [
      [16, 31, 17, 4], [18, 24, 13, 8], [21, 18, 7, 8], [23, 14, 3, 6],
    ])
    pixels(drawing, PALETTE.blue, [[19, 28, 11, 4], [21, 23, 7, 6], [23, 18, 3, 7]])
    pixels(drawing, PALETTE.blueLight, [[23, 25, 3, 4], [24, 20, 2, 4]])
    pixels(drawing, PALETTE.brick, [[14, 15, 4, 4], [31, 15, 4, 4], [13, 34, 23, 3]])
  } else if (kind === 'longFuse') {
    pixels(drawing, PALETTE.deepInk, [[11, 24, 22, 12], [15, 20, 14, 18], [28, 16, 5, 10], [31, 12, 9, 5]])
    pixels(drawing, PALETTE.ink, [[14, 25, 16, 9], [18, 22, 9, 4]])
    pixels(drawing, PALETTE.blueLight, [[16, 26, 4, 5]])
    pixels(drawing, PALETTE.flame, [[39, 9, 4, 5], [42, 13, 4, 4]])
    pixels(drawing, PALETTE.flameCore, [[40, 11, 3, 3]])
  } else if (kind === 'reverseControls') {
    pixels(drawing, PALETTE.deepInk, [
      [12, 17, 22, 4], [12, 29, 22, 4], [9, 20, 4, 10], [33, 20, 4, 10],
      [30, 29, 7, 4],
    ])
    pixels(drawing, PALETTE.blue, [[13, 21, 19, 4], [13, 25, 19, 4]])
    pixels(drawing, PALETTE.white, [[9, 21, 7, 4], [30, 25, 7, 4]])
    pixels(drawing, PALETTE.brick, [[9, 25, 4, 4], [33, 21, 4, 4]])
  } else if (kind === 'invisibleBombs') {
    pixels(drawing, PALETTE.deepInk, [
      [15, 22, 19, 13], [19, 18, 11, 20], [29, 15, 4, 8], [32, 12, 5, 5],
      [12, 16, 25, 3], [12, 34, 25, 3],
    ])
    pixels(drawing, PALETTE.ink, [[18, 23, 13, 10], [22, 20, 6, 5]])
    pixels(drawing, PALETTE.blue, [[11, 20, 5, 11], [33, 20, 5, 11]])
    pixels(drawing, PALETTE.blueLight, [[12, 22, 3, 7], [34, 22, 3, 7]])
    pixels(drawing, PALETTE.white, [[9, 14, 4, 4], [37, 32, 4, 4]])
  } else {
    pixels(drawing, PALETTE.deepInk, [
      [12, 17, 25, 4], [9, 21, 7, 10], [33, 21, 7, 10], [12, 31, 25, 4],
    ])
    pixels(drawing, PALETTE.blue, [
      [15, 19, 19, 4], [12, 23, 7, 6], [30, 23, 7, 6], [15, 29, 19, 4],
    ])
    pixels(drawing, PALETTE.blueLight, [[18, 20, 13, 3], [14, 24, 3, 4], [32, 24, 3, 4]])
    pixels(drawing, PALETTE.flame, [[22, 22, 5, 9], [19, 25, 11, 3]])
    pixels(drawing, PALETTE.white, [[23, 23, 3, 7]])
  }
  return drawing
}

export function makePickup(kind: RenderablePickupKind): Graphics {
  return kind === 'kick' ? makeKickPickup()
    : kind === 'punch' ? makePunchPickup()
    : kind === 'bracelet' ? makeBraceletPickup()
    : kind === 'wallPass' ? makeWallPassPickup() : makeLegacyPickup(kind)
}
