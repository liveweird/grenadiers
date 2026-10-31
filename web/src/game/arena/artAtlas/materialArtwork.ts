import { Graphics } from 'pixi.js'

import type { MATERIAL_FRAME_IDS } from './descriptor'

export type MaterialFrameId = typeof MATERIAL_FRAME_IDS[number]

const COLOR = {
  sandstone: 0xd8c69d,
  sandstoneLight: 0xeee0b9,
  sandstoneShade: 0xad9870,
  field: 0x879b76,
  fieldLight: 0xa8b994,
  fieldShade: 0x65785c,
  brick: 0xb55336,
  brickLight: 0xd87951,
  brickShade: 0x7e372d,
  slate: 0x3e4e59,
  slateLight: 0x62727c,
  slateShade: 0x303c47,
  gold: 0xffc462,
  ink: 0x263445,
} as const

type PixelRect = readonly [x: number, y: number, width: number, height: number]

function paint(graphics: Graphics, color: number, rectangles: readonly PixelRect[]): void {
  for (const [x, y, width, height] of rectangles) graphics.rect(x, y, width, height).fill(color)
}

function floorTop(alternate: boolean): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.field)
  paint(art, COLOR.fieldLight, alternate
    ? [[4, 7, 12, 3], [31, 4, 9, 3], [23, 25, 14, 3], [7, 38, 16, 3]]
    : [[7, 5, 15, 3], [34, 12, 8, 3], [5, 27, 12, 3], [27, 39, 14, 3]])
  paint(art, COLOR.fieldShade, alternate
    ? [[10, 10, 3, 6], [35, 7, 3, 8], [27, 28, 3, 7], [12, 34, 3, 4]]
    : [[12, 8, 3, 7], [38, 15, 3, 6], [9, 30, 3, 6], [32, 35, 3, 7]])
  paint(art, COLOR.sandstone, alternate ? [[2, 21, 8, 3], [39, 34, 6, 3]] : [[24, 17, 7, 3], [3, 42, 9, 3]])
  return art
}

function wallTop(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.sandstone)
  paint(art, COLOR.sandstoneShade, [
    [0, 14, 48, 3], [0, 31, 48, 3], [14, 0, 3, 14], [34, 0, 3, 14],
    [7, 17, 3, 14], [27, 17, 3, 14], [18, 34, 3, 14], [39, 34, 3, 14],
  ])
  paint(art, COLOR.sandstoneLight, [[3, 3, 9, 3], [20, 20, 6, 3], [32, 37, 5, 3]])
  return art
}

function wallSide(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.sandstoneShade)
  paint(art, COLOR.sandstone, [
    [0, 3, 22, 10], [25, 3, 23, 10], [0, 16, 12, 11], [15, 16, 22, 11], [40, 16, 8, 11],
    [0, 30, 22, 12], [25, 30, 23, 12],
  ])
  paint(art, COLOR.sandstoneLight, [[2, 5, 18, 3], [27, 5, 18, 3], [17, 18, 17, 3], [27, 32, 18, 3]])
  paint(art, COLOR.ink, [[0, 45, 48, 3]])
  return art
}

function blockTop(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.brickShade)
  paint(art, COLOR.brick, [
    [2, 2, 20, 12], [25, 2, 21, 12], [2, 17, 11, 12], [16, 17, 21, 12], [40, 17, 6, 12],
    [2, 32, 20, 14], [25, 32, 21, 14],
  ])
  paint(art, COLOR.brickLight, [[4, 4, 16, 3], [27, 4, 17, 3], [18, 19, 17, 3], [27, 34, 17, 3]])
  return art
}

function blockSide(): Graphics {
  const art = blockTop()
  paint(art, COLOR.ink, [[0, 0, 48, 3], [0, 45, 48, 3]])
  paint(art, COLOR.brickShade, [[6, 37, 3, 8], [34, 22, 3, 7]])
  return art
}

function lairTop(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.slate)
  paint(art, COLOR.slateLight, [[4, 4, 40, 4], [4, 40, 40, 4], [4, 8, 4, 32], [40, 8, 4, 32]])
  paint(art, COLOR.ink, [[10, 10, 28, 28], [7, 19, 34, 10], [19, 7, 10, 34]])
  paint(art, COLOR.gold, [[21, 11, 6, 8], [21, 29, 6, 8], [11, 21, 8, 6], [29, 21, 8, 6]])
  paint(art, COLOR.slateShade, [[16, 16, 16, 16]])
  return art
}

function lairSide(): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(COLOR.slateShade)
  paint(art, COLOR.slate, [[3, 3, 42, 11], [3, 17, 42, 11], [3, 31, 42, 14]])
  paint(art, COLOR.slateLight, [[5, 5, 38, 3], [5, 19, 38, 3], [5, 33, 38, 3]])
  paint(art, COLOR.gold, [[9, 23, 6, 5], [33, 9, 6, 5], [22, 37, 5, 8]])
  paint(art, COLOR.ink, [[0, 45, 48, 3]])
  return art
}

function rim(top: boolean): Graphics {
  const art = new Graphics().rect(0, 0, 48, 48).fill(top ? COLOR.slate : COLOR.slateShade)
  paint(art, COLOR.ink, top
    ? [[0, 0, 48, 4], [0, 44, 48, 4], [0, 4, 4, 40], [44, 4, 4, 40]]
    : [[0, 0, 48, 4], [0, 44, 48, 4], [11, 4, 4, 40], [33, 4, 4, 40]])
  paint(art, COLOR.slateLight, top
    ? [[7, 7, 34, 4], [7, 37, 34, 4]]
    : [[3, 7, 8, 4], [15, 7, 18, 4], [37, 7, 8, 4]])
  paint(art, COLOR.gold, top
    ? [[8, 20, 8, 8], [32, 20, 8, 8]]
    : [[4, 22, 7, 5], [37, 22, 7, 5]])
  return art
}

export function makeMaterialFrame(id: MaterialFrameId): Graphics {
  switch (id) {
    case 'material.floor.top.0': return floorTop(false)
    case 'material.floor.top.1': return floorTop(true)
    case 'material.wall.top': return wallTop()
    case 'material.wall.side': return wallSide()
    case 'material.block.top': return blockTop()
    case 'material.block.side': return blockSide()
    case 'material.lair.top': return lairTop()
    case 'material.lair.side': return lairSide()
    case 'material.rim.top': return rim(true)
    case 'material.rim.side': return rim(false)
  }
}
