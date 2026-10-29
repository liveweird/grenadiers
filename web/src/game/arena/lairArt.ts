import { Graphics } from 'pixi.js'
import { pixels } from './pixel'
import type { Direction } from './types'

/** Procedural authoring reference; gameplay imports the adopted pixel atlas. */
export function makeLair(): Graphics {
  const drawing = new Graphics()
  pixels(drawing, 0x302a27, [[0, 0, 48, 48]])
  pixels(drawing, 0xb39355, [[2, 2, 44, 44]])
  pixels(drawing, 0xe7cd8e, [
    [3, 3, 19, 8], [25, 3, 20, 8], [3, 14, 8, 13], [37, 14, 8, 13],
    [3, 30, 8, 11], [37, 30, 8, 11], [14, 39, 20, 6],
  ])
  pixels(drawing, 0xffe7a9, [[4, 3, 17, 2], [26, 3, 17, 2], [3, 15, 2, 10]])
  pixels(drawing, 0x171a23, [[12, 18, 24, 23], [16, 13, 16, 5], [20, 10, 8, 3]])
  pixels(drawing, 0x5a5a60, [[17, 17, 2, 24], [28, 17, 2, 24], [12, 27, 24, 2]])
  pixels(drawing, 0x9e8e67, [[17, 18, 1, 22], [28, 18, 1, 22], [13, 27, 22, 1]])
  pixels(drawing, 0xf6b431, [[22, 30, 2, 2], [26, 30, 2, 2]])
  pixels(drawing, 0x586241, [[3, 39, 8, 4], [6, 36, 3, 3], [37, 39, 6, 3]])
  return drawing
}

export function makeMob(direction: Direction): Graphics {
  const drawing = new Graphics()
  const ink = 0x151827
  pixels(drawing, ink, [[-13, -12, 26, 17], [-18, 1, 36, 12], [-12, 10, 24, 10]])
  pixels(drawing, 0x77777e, [[-10, -11, 20, 15], [-14, 3, 28, 10], [-10, 12, 20, 6]])
  pixels(drawing, 0xb0a99a, [[-8, -11, 11, 3], [-12, 4, 5, 4], [8, 4, 4, 3]])
  pixels(drawing, 0xe0c796, [[-11, -12, 4, 6], [7, -12, 4, 6]])
  pixels(drawing, 0x7d3028, [[-14, 1, 28, 5], [-10, 6, 20, 3]])
  pixels(drawing, 0xb45033, [[-13, 1, 26, 2], [-9, 5, 18, 2]])
  pixels(drawing, ink, [[-12, 17, 9, 4], [3, 17, 9, 4]])
  pixels(drawing, 0xd4b985, [[-11, 18, 7, 2], [4, 18, 7, 2]])
  if (direction === 'north') {
    pixels(drawing, 0x9e3d2b, [[-6, 4, 5, 10], [1, 4, 5, 8]])
  } else if (direction === 'south') {
    pixels(drawing, ink, [[-9, -6, 18, 7]])
    pixels(drawing, 0xe8a128, [[-7, -5, 4, 3], [3, -5, 4, 3]])
    pixels(drawing, 0xffe7a9, [[-6, -5, 1, 1], [4, -5, 1, 1]])
  } else {
    const sign = direction === 'east' ? 1 : -1
    pixels(drawing, ink, [[sign === 1 ? 1 : -11, -6, 10, 7]])
    pixels(drawing, 0xe8a128, [[sign === 1 ? 5 : -9, -5, 4, 3]])
    pixels(drawing, 0xffe7a9, [[sign === 1 ? 7 : -8, -5, 1, 1]])
  }
  return drawing
}
