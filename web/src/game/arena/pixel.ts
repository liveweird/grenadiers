import type { Graphics } from 'pixi.js'

export type PixelRect = readonly [x: number, y: number, width: number, height: number]

export function pixels(graphics: Graphics, color: number, rectangles: readonly PixelRect[]): Graphics {
  for (const [x, y, width, height] of rectangles) graphics.rect(x, y, width, height).fill(color)
  return graphics
}

export function deterministicVariant(column: number, row: number, variants: number): number {
  return ((column * 17 + row * 31 + column * row * 7) >>> 0) % variants
}
