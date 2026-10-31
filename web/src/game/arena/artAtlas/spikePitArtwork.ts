import { Graphics } from 'pixi.js'

export type SpikePitArtworkPhase = 'idle' | 'warning' | 'raised'

type PixelRect = readonly [x: number, y: number, width: number, height: number]

const COLOR = {
  sand: 0xc8b486,
  sandLight: 0xe0d0a4,
  sandShade: 0x9c865e,
  iron: 0x59616a,
  ironLight: 0x929aa0,
  ironShade: 0x343b43,
  pit: 0x20272d,
  silver: 0xc9d0d2,
  silverLight: 0xf0f1e8,
  amber: 0xe6a72e,
  amberLight: 0xffd45c,
} as const

function paint(graphics: Graphics, color: number, rectangles: readonly PixelRect[]): void {
  for (const [x, y, width, height] of rectangles) graphics.rect(x, y, width, height).fill(color)
}

function pitBase(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.sand)
  paint(art, COLOR.sandLight, [[3, 3, 42, 3], [3, 6, 3, 36]])
  paint(art, COLOR.sandShade, [[3, 42, 42, 3], [42, 6, 3, 36]])
  paint(art, COLOR.ironShade, [[7, 7, 34, 34]])
  paint(art, COLOR.iron, [[9, 9, 30, 3], [9, 36, 30, 3], [9, 12, 3, 24], [36, 12, 3, 24]])
  paint(art, COLOR.pit, [[12, 12, 24, 24]])
  return art
}

function retractedMechanism(art: Graphics): void {
  paint(art, COLOR.ironShade, [[15, 16, 18, 4], [15, 28, 18, 4]])
  paint(art, COLOR.iron, [[14, 20, 20, 8]])
  paint(art, COLOR.ironLight, [[16, 21, 16, 2], [16, 26, 16, 2]])
  paint(art, COLOR.silver, [[19, 23, 3, 3], [26, 23, 3, 3]])
}

function warningBorder(art: Graphics): void {
  paint(art, COLOR.amber, [
    [4, 4, 40, 3], [4, 41, 40, 3], [4, 7, 3, 34], [41, 7, 3, 34],
  ])
  paint(art, COLOR.amberLight, [
    [8, 4, 6, 3], [20, 4, 6, 3], [32, 4, 6, 3],
    [8, 41, 6, 3], [20, 41, 6, 3], [32, 41, 6, 3],
    [4, 14, 3, 6], [4, 26, 3, 6], [41, 14, 3, 6], [41, 26, 3, 6],
  ])
  // A centered exclamation marker keeps warning readable without relying on hue.
  paint(art, COLOR.amberLight, [[22, 13, 4, 9], [22, 25, 4, 4]])
}

function raisedSpikes(art: Graphics): void {
  paint(art, COLOR.iron, [[13, 31, 22, 4]])
  const spikes = [14, 20, 26, 32]
  for (const x of spikes) {
    art.poly([x, 31, x + 3, 12, x + 6, 31]).fill(COLOR.silver)
    art.poly([x + 3, 12, x + 5, 31, x + 6, 31]).fill(COLOR.ironLight)
    art.poly([x + 3, 12, x + 4, 19, x + 2, 19]).fill(COLOR.silverLight)
  }
}

/** Procedural authoring reference; gameplay imports the adopted artist atlas. */
export function makeSpikePit(phase: SpikePitArtworkPhase): Graphics {
  const art = pitBase()
  if (phase === 'raised') raisedSpikes(art)
  else {
    retractedMechanism(art)
    if (phase === 'warning') warningBorder(art)
  }
  return art
}
