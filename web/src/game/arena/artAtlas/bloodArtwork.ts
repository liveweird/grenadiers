import { Graphics } from 'pixi.js'

type PixelRect = readonly [x: number, y: number, width: number, height: number]

function paint(graphics: Graphics, color: number, rectangles: readonly PixelRect[]): void {
  for (const [x, y, width, height] of rectangles) graphics.rect(x, y, width, height).fill(color)
}

/** Procedural authoring reference; gameplay imports the adopted artist atlas. */
export function makeBloodPool(): Graphics {
  const drawing = new Graphics()
  paint(drawing, 0x3b1117, [
    [16, 15, 16, 2], [13, 17, 23, 2], [10, 19, 28, 10],
    [12, 29, 24, 2], [16, 31, 16, 2],
  ])
  paint(drawing, 0x62151b, [
    [15, 18, 18, 2], [12, 20, 24, 8], [15, 28, 18, 2],
  ])
  paint(drawing, 0x841e22, [
    [16, 20, 15, 3], [14, 23, 9, 4], [26, 24, 8, 3], [20, 27, 11, 2],
  ])
  paint(drawing, 0xa52b2b, [[20, 21, 8, 2], [16, 24, 5, 2]])
  return drawing
}
