import { Graphics } from 'pixi.js'

export type FireArtworkId =
  | 'effect.flame.2' | 'effect.flame.3' | 'effect.flame.4' | 'effect.flame.5'
  | 'effect.ember' | 'effect.smoke' | 'effect.ignition'

type PixelRect = readonly [x: number, y: number, width: number, height: number]

const COLOR = {
  cream: 0xfff1ad,
  amber: 0xffcf4a,
  orange: 0xf47a2d,
  copper: 0xa94325,
  smokeLight: 0xc9c1b2,
  smoke: 0x918b83,
  smokeShade: 0x5d5b5c,
} as const

function paint(graphics: Graphics, color: number, rectangles: readonly PixelRect[]): void {
  for (const [x, y, width, height] of rectangles) graphics.rect(x, y, width, height).fill(color)
}

function flame(pose: 2 | 3 | 4 | 5): Graphics {
  const art = new Graphics()
  // The 22px copper/orange cross reaches every edge, joining adjacent flame tiles.
  paint(art, COLOR.copper, [[0, 13, 48, 22], [13, 0, 22, 48]])
  paint(art, COLOR.orange, pose === 2 ? [
    [0, 17, 48, 14], [17, 0, 14, 48], [8, 12, 32, 24],
  ] : pose === 3 ? [
    [0, 15, 48, 16], [15, 0, 16, 48], [10, 9, 28, 30],
  ] : pose === 4 ? [
    [0, 18, 48, 13], [18, 0, 13, 48], [6, 14, 36, 21],
  ] : [
    [0, 16, 48, 15], [16, 0, 15, 48], [11, 7, 26, 34],
  ])
  paint(art, COLOR.amber, pose === 2 ? [
    [6, 20, 36, 9], [20, 6, 9, 36], [13, 14, 22, 20],
  ] : pose === 3 ? [
    [8, 18, 32, 10], [18, 4, 10, 39], [13, 11, 21, 27],
  ] : pose === 4 ? [
    [4, 21, 40, 8], [21, 7, 8, 35], [12, 15, 25, 19],
  ] : [
    [7, 19, 34, 10], [19, 3, 10, 41], [14, 10, 20, 28],
  ])
  paint(art, COLOR.cream, pose === 2 ? [[18, 20, 13, 9], [22, 13, 5, 22]]
    : pose === 3 ? [[17, 19, 14, 9], [22, 10, 5, 24]]
      : pose === 4 ? [[16, 21, 16, 7], [22, 15, 5, 19]]
        : [[18, 18, 12, 11], [22, 8, 5, 27]])
  return art
}

function ember(): Graphics {
  return new Graphics().rect(22, 22, 4, 4).fill(COLOR.amber)
}

function smoke(): Graphics {
  const art = new Graphics()
  paint(art, COLOR.smokeShade, [[13, 21, 25, 13], [17, 14, 17, 24], [21, 9, 10, 29]])
  paint(art, COLOR.smoke, [[10, 23, 28, 8], [15, 16, 20, 14], [20, 11, 12, 20]])
  paint(art, COLOR.smokeLight, [[17, 18, 13, 7], [22, 13, 7, 12], [13, 24, 9, 4]])
  return art
}

function ignition(): Graphics {
  const art = new Graphics()
  paint(art, COLOR.copper, [[22, 5, 4, 38], [5, 22, 38, 4]])
  paint(art, COLOR.orange, [[14, 14, 20, 20], [10, 20, 28, 8], [20, 10, 8, 28]])
  paint(art, COLOR.amber, [[17, 17, 14, 14], [13, 21, 22, 6], [21, 13, 6, 22]])
  paint(art, COLOR.cream, [[20, 20, 8, 8], [22, 16, 4, 16], [16, 22, 16, 4]])
  return art
}

/** Procedural authoring reference; gameplay imports the adopted artist atlas. */
export function makeFireArtwork(id: FireArtworkId): Graphics {
  switch (id) {
    case 'effect.flame.2': return flame(2)
    case 'effect.flame.3': return flame(3)
    case 'effect.flame.4': return flame(4)
    case 'effect.flame.5': return flame(5)
    case 'effect.ember': return ember()
    case 'effect.smoke': return smoke()
    case 'effect.ignition': return ignition()
  }
}
