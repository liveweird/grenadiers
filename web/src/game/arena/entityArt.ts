import { Container, Graphics } from 'pixi.js'
import type { RenderablePickupKind } from './types'
import { PALETTE } from './palette'
import { pixels } from './pixel'

export interface BombDrawing { root: Container; spark: Graphics }
export interface FlameDrawing { root: Container; frames: readonly [Graphics, Graphics] }

export function makeBomb(): BombDrawing {
  const root = new Container()
  const shadow = new Graphics()
  pixels(shadow, PALETTE.deepInk, [[-15, 12, 30, 5], [-10, 17, 20, 2]])
  shadow.alpha = 0.28
  const shell = new Graphics()
  pixels(shell, PALETTE.deepInk, [[-12, -9, 24, 21], [-9, -13, 18, 29], [-15, -3, 30, 12]])
  pixels(shell, PALETTE.ink, [[-9, -9, 18, 21], [-12, -3, 24, 12]])
  pixels(shell, 0x536272, [[-7, -9, 6, 9], [-10, -3, 6, 5]])
  pixels(shell, 0x8796a0, [[-6, -8, 3, 4]])
  pixels(shell, PALETTE.brass, [[4, -15, 7, 5], [8, -19, 4, 7], [11, -21, 7, 3]])
  const spark = new Graphics()
  pixels(spark, PALETTE.flameCore, [[16, -25, 4, 4], [21, -21, 4, 4], [12, -21, 4, 4]])
  pixels(spark, PALETTE.flame, [[18, -29, 3, 4], [24, -24, 3, 3]])
  root.addChild(shadow, shell, spark)
  return { root, spark }
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

export function makePickup(kind: RenderablePickupKind): Graphics {
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
  } else if (kind === 'shield') {
    pixels(drawing, PALETTE.deepInk, [
      [16, 15, 17, 3], [13, 18, 23, 7], [15, 25, 19, 7], [18, 32, 13, 4], [22, 36, 5, 3],
    ])
    pixels(drawing, PALETTE.blue, [
      [17, 18, 15, 4], [16, 22, 17, 6], [18, 28, 13, 4], [21, 32, 7, 3],
    ])
    pixels(drawing, PALETTE.blueLight, [[18, 19, 9, 3], [18, 22, 4, 5]])
  } else {
    pixels(drawing, PALETTE.deepInk, [
      [16, 32, 17, 4], [13, 25, 23, 7], [17, 17, 15, 10], [22, 12, 7, 8],
    ])
    pixels(drawing, PALETTE.flame, [[16, 25, 17, 6], [19, 18, 11, 10], [23, 13, 5, 9]])
    pixels(drawing, PALETTE.flameLight, [[20, 24, 9, 5], [22, 18, 6, 9], [24, 15, 3, 7]])
    pixels(drawing, PALETTE.flameCore, [[23, 22, 4, 7], [24, 18, 2, 5]])
  }
  return drawing
}
